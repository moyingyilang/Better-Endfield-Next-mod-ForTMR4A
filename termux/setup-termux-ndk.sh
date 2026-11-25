#!/data/data/com.termux/files/usr/bin/sh
# Set up an aarch64-Termux-compatible NDK shim for local Android builds.
#
# The NDK ships x86_64 host tools only. This keeps their data (sysroot, headers,
# libraries) and replaces the host executables with:
#   * native arm64 Termux clang/llvm-* (used for nearly everything);
#   * box64-wrapped x86_64 tools for the few with no arm64 equivalent.
# It also creates a CMake shim (box64 cmake + native ninja) because AGP always
# launches CMake as a host tool.
#
# Usage (inside Termux):
#   termux/setup-termux-ndk.sh
#   # then:
#   export BE_TERMUX_NDK="$TOOLCHAIN/ndk-compat"
#   printf 'sdk.dir=%s\ncmake.dir=%s\n' "$SDK" "$TOOLCHAIN/android-cmake" > android/local.properties
#
# Environment overrides:
#   BE_ANDROID_SDK        Android SDK root (default: $ANDROID_SDK_ROOT or ~/android-sdk)
#   BE_NDK_VERSION        NDK version (default: 27.2.12479018)
#   BE_TERMUX_TOOLCHAIN   output root (default: ~/.cache/better-endfield-next/termux-ndk)
#   BE_BOX64              box64 binary (default: $PREFIX/glibc/bin/box64 or $PREFIX/bin/box64)
set -eu

PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
SDK="${BE_ANDROID_SDK:-${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/android-sdk}}}"
NDK_VERSION="${BE_NDK_VERSION:-27.2.12479018}"
OUT="${BE_TERMUX_TOOLCHAIN:-$HOME/.cache/better-endfield-next/termux-ndk}"
NDK="$SDK/ndk/$NDK_VERSION"
PRE=linux-x86_64
REAL_PRE="$NDK/toolchains/llvm/prebuilt/$PRE"
COMPAT="$OUT/ndk-compat"
COMPAT_PRE="$COMPAT/toolchains/llvm/prebuilt/$PRE"
SYSROOT="$COMPAT_PRE/sysroot"

[ -d "$NDK" ] || { echo "NDK $NDK_VERSION not found under $SDK" >&2; exit 1; }

# box64.
if [ -n "${BE_BOX64:-}" ]; then
    BOX64="$BE_BOX64"
elif [ -x "$PREFIX/glibc/bin/box64" ]; then
    BOX64="$PREFIX/glibc/bin/box64"
elif [ -x "$PREFIX/bin/box64" ]; then
    BOX64="$PREFIX/bin/box64"
else
    echo "box64 not found; install box64-glibc or set BE_BOX64" >&2
    exit 1
fi

# Deterministic clang version directory (e.g. 18, 21).
CLANG_VER=$(ls "$REAL_PRE/lib/clang" | sort | tail -n 1)
RES_AARCH64="$REAL_PRE/lib/clang/$CLANG_VER/lib/linux/aarch64"
# API-level lib dir first: it carries the shared stub libs (libc.so ...), so
# `-lc` resolves the shared library instead of falling into libc.a (gwp_asan).
LINK_FLAGS="-L\"$SYSROOT/usr/lib/aarch64-linux-android/29\" -L\"$SYSROOT/usr/lib/aarch64-linux-android\" -L\"$RES_AARCH64\""

rm -rf "$COMPAT"
mkdir -p "$COMPAT_PRE"
cp "$NDK/source.properties" "$COMPAT/source.properties"
cp -R "$NDK/build" "$COMPAT/build"
[ -d "$NDK/meta" ] && cp -R "$NDK/meta" "$COMPAT/meta"

for item in "$REAL_PRE"/*; do
    name=$(basename "$item")
    case "$name" in
        bin|lib|sysroot) ;;
        *) ln -s "$item" "$COMPAT_PRE/$name" ;;
    esac
done
ln -s "$REAL_PRE/lib" "$COMPAT_PRE/lib"
ln -s "$REAL_PRE/sysroot" "$COMPAT_PRE/sysroot"

BIN="$COMPAT_PRE/bin"
mkdir -p "$BIN"
REAL_BIN="$REAL_PRE/bin"

native_tool() {
    case "$1" in
        llvm-ar|llvm-ranlib|llvm-strip|llvm-objcopy|llvm-nm|llvm-readelf|llvm-objdump|llvm-size|llvm-addr2line|llvm-cxxfilt|llvm-symbolizer|ld.lld|lld)
            printf '%s' "$1" ;;
        *) return 1 ;;
    esac
}

for item in "$REAL_BIN"/*; do
    name=$(basename "$item")
    wrapper="$BIN/$name"
    case "$name" in
        *clang*)
            case "$name" in
                *++*) compiler=clang++ ;;
                *) compiler=clang ;;
            esac
            if [ "$compiler" = clang++ ]; then
                cat > "$wrapper" <<WRAP
#!/data/data/com.termux/files/usr/bin/sh
link=1
for a in "\$@"; do
  case "\$a" in -c|-E|-S|-M|-MM|-MD|-MMD|-###|-v|--version|-dumpversion) link=0 ;; esac
done
if [ "\$link" = 1 ]; then
  exec "$PREFIX/bin/clang++" --sysroot="$SYSROOT" -nostdinc++ -isystem "$SYSROOT/usr/include/c++/v1" $LINK_FLAGS "\$@" -nostdlib++ -lc++_static -lc++abi -lunwind --target=aarch64-linux-android29
else
  exec "$PREFIX/bin/clang++" --sysroot="$SYSROOT" -nostdinc++ -isystem "$SYSROOT/usr/include/c++/v1" "\$@" --target=aarch64-linux-android29
fi
WRAP
            else
                cat > "$wrapper" <<WRAP
#!/data/data/com.termux/files/usr/bin/sh
link=1
for a in "\$@"; do
  case "\$a" in -c|-E|-S|-M|-MM|-MD|-MMD|-###|-v|--version|-dumpversion) link=0 ;; esac
done
if [ "\$link" = 1 ]; then
  exec "$PREFIX/bin/clang" --sysroot="$SYSROOT" $LINK_FLAGS "\$@" --target=aarch64-linux-android29
else
  exec "$PREFIX/bin/clang" --sysroot="$SYSROOT" "\$@" --target=aarch64-linux-android29
fi
WRAP
            fi
            ;;
        *)
            if native_tool "$name" >/dev/null 2>&1; then
                printf '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/bin/%s" "$@"\n' \
                    "$PREFIX" "$(native_tool "$name")" > "$wrapper"
            else
                printf '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/box64-x86_64.sh" "%s/%s" "$@"\n' \
                    "$OUT" "$REAL_BIN" "$name" > "$wrapper"
            fi
            ;;
    esac
    chmod 755 "$wrapper"
done

# box64 wrapper (glibc runtime, no binfmt registration).
cat > "$OUT/box64-x86_64.sh" <<WRAP
#!/data/data/com.termux/files/usr/bin/sh
unset LD_PRELOAD
exec "$PREFIX/glibc/lib/ld-linux-aarch64.so.1" --library-path "$PREFIX/glibc/lib" --inhibit-cache "$BOX64" "\$@"
WRAP
chmod 755 "$OUT/box64-x86_64.sh"

# Patch the NDK toolchain: force the x86_64 host tag + put the wrapper bin on -B.
TOOLCHAIN="$COMPAT/build/cmake/android.toolchain.cmake"
awk '
    /^# Toolchain\.$/ && !done {
        print "if(NOT ANDROID_HOST_TAG)"
        print "  set(ANDROID_HOST_TAG linux-x86_64)"
        print "endif()"
        print ""
        done = 1
    }
    { print }
' "$TOOLCHAIN" > "$TOOLCHAIN.new"
mv "$TOOLCHAIN.new" "$TOOLCHAIN"
awk '
    /^set\(CMAKE_SYSROOT "\$\{ANDROID_TOOLCHAIN_ROOT\}\/sysroot"\)$/ {
        print
        print "set(CMAKE_C_FLAGS_INIT \"${CMAKE_C_FLAGS_INIT} -B${ANDROID_TOOLCHAIN_ROOT}/bin\")"
        print "set(CMAKE_CXX_FLAGS_INIT \"${CMAKE_CXX_FLAGS_INIT} -B${ANDROID_TOOLCHAIN_ROOT}/bin\")"
        next
    }
    { print }
' "$TOOLCHAIN" > "$TOOLCHAIN.new"
mv "$TOOLCHAIN.new" "$TOOLCHAIN"

# CMake/ninja shim for AGP (cmake host tool goes through box64, ninja is native).
SHIM="$OUT/android-cmake/bin"
mkdir -p "$SHIM"
printf '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/box64-x86_64.sh" "%s/cmake/3.22.1/bin/cmake" "$@"\n' \
    "$OUT" "$SDK" > "$SHIM/cmake"
printf '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/bin/ninja" "$@"\n' "$PREFIX" > "$SHIM/ninja"
chmod 755 "$SHIM/cmake" "$SHIM/ninja"

echo "compat NDK : $COMPAT"
echo "cmake shim : $SHIM"
echo
echo "export BE_TERMUX_NDK=$COMPAT"
printf 'printf "sdk.dir=%%s\\\\ncmake.dir=%%s\\\\n" "%s" "%s" > android/local.properties\n' \
    "$SDK" "${SHIM%/*}"
