#!/usr/bin/env python3
"""Set up an aarch64-Termux-compatible NDK shim for local Android builds.

The NDK ships x86_64 host tools only. This script keeps their data (sysroot,
headers, libraries) and replaces the host executables with:

  * native arm64 Termux clang/llvm-* (fast path, used for nearly everything);
  * box64-wrapped x86_64 tools for the few that have no arm64 equivalent.

It also creates a CMake shim (box64 cmake + native ninja) because AGP always
launches CMake as a host tool.

Usage (inside Termux):

    python termux/setup-termux-ndk.py
    # then build with:
    #   BE_TERMUX_NDK=<out>/ndk-compat ./gradlew :app:assembleDebug ...
    #   cmake.dir=<out>/android-cmake in android/local.properties
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path


DEFAULT_NDK_VERSION = "27.2.12479018"


def termux_prefix() -> Path:
    return Path(os.environ.get("PREFIX", "/data/data/com.termux/files/usr"))


def find_box64() -> Path:
    prefix = termux_prefix()
    for candidate in (
        prefix / "glibc/bin/box64",
        prefix / "bin/box64",
    ):
        if candidate.is_file():
            return candidate
    raise SystemExit(
        "box64 not found; install the Termux package that provides it "
        "(e.g. box64-glibc) or pass --box64"
    )


def write_box64_wrapper(out_root: Path, box64: Path) -> Path:
    prefix = termux_prefix()
    wrapper = out_root / "box64-x86_64.sh"
    wrapper.write_text(
        "#!/data/data/com.termux/files/usr/bin/sh\n"
        "# x86_64 ELF -> box64 (glibc runtime). No binfmt registration anywhere.\n"
        "unset LD_PRELOAD\n"
        'exec "%s/glibc/lib/ld-linux-aarch64.so.1" '
        '--library-path "%s/glibc/lib" --inhibit-cache '
        '"%s" "$@"\n' % (prefix, prefix, box64),
        encoding="utf-8",
    )
    wrapper.chmod(0o755)
    return wrapper


def write_cmake_shim(out_root: Path, sdk: Path, prefix: Path) -> Path:
    shim = out_root / "android-cmake/bin"
    shim.mkdir(parents=True, exist_ok=True)
    sdk_cmake = sdk / "cmake/3.22.1/bin/cmake"
    box = out_root / "box64-x86_64.sh"
    (shim / "cmake").write_text(
        "#!/data/data/com.termux/files/usr/bin/sh\n"
        'exec "%s" "%s" "$@"\n' % (box, sdk_cmake),
        encoding="utf-8",
    )
    (shim / "ninja").write_text(
        '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/bin/ninja" "$@"\n' % prefix,
        encoding="utf-8",
    )
    for entry in shim.iterdir():
        entry.chmod(0o755)
    return shim


NATIVE_TOOLS = {
    "llvm-ar": "llvm-ar",
    "llvm-ranlib": "llvm-ranlib",
    "llvm-strip": "llvm-strip",
    "llvm-objcopy": "llvm-objcopy",
    "llvm-nm": "llvm-nm",
    "llvm-readelf": "llvm-readelf",
    "llvm-objdump": "llvm-objdump",
    "llvm-size": "llvm-size",
    "llvm-addr2line": "llvm-addr2line",
    "llvm-cxxfilt": "llvm-cxxfilt",
    "llvm-symbolizer": "llvm-symbolizer",
    "ld.lld": "ld.lld",
    "lld": "lld",
}


def make_compat_ndk(out_root: Path, ndk: Path, prefix: Path) -> Path:
    import shutil

    compat = out_root / "ndk-compat"
    pre = "linux-x86_64"
    real_pre = ndk / "toolchains/llvm/prebuilt" / pre
    compat_pre = compat / "toolchains/llvm/prebuilt" / pre
    sysroot = compat_pre / "sysroot"
    clang_ver = sorted(p.name for p in (real_pre / "lib/clang").iterdir())[-1]
    res_aarch64 = real_pre / "lib/clang" / clang_ver / "lib/linux/aarch64"

    shutil.rmtree(compat, ignore_errors=True)
    compat_pre.mkdir(parents=True)
    shutil.copy2(ndk / "source.properties", compat / "source.properties")
    shutil.copytree(ndk / "build", compat / "build")
    if (ndk / "meta").is_dir():
        shutil.copytree(ndk / "meta", compat / "meta")
    for child in real_pre.iterdir():
        if child.name in ("bin", "lib", "sysroot"):
            continue
        (compat_pre / child.name).symlink_to(child)
    (compat_pre / "lib").symlink_to(real_pre / "lib", target_is_directory=True)
    (compat_pre / "sysroot").symlink_to(real_pre / "sysroot", target_is_directory=True)

    box = out_root / "box64-x86_64.sh"
    bin_dir = prefix / "bin"
    link_flags = '-L"%s/usr/lib/aarch64-linux-android" -L"%s"' % (sysroot, res_aarch64)
    wrap_bin = compat_pre / "bin"
    wrap_bin.mkdir()
    real_bin = real_pre / "bin"
    for name in sorted(os.listdir(real_bin)):
        path = wrap_bin / name
        if "clang" in name:
            if "++" in name:
                text = (
                    "#!/data/data/com.termux/files/usr/bin/sh\n"
                    "link=1\n"
                    'for a in "$@"; do\n'
                    '  case "$a" in -c|-E|-S|-M|-MM|-MD|-MMD|-###|-v|--version|-dumpversion) link=0 ;; esac\n'
                    "done\n"
                    'if [ "$link" = 1 ]; then\n'
                    '  exec "%s/clang++" --sysroot="%s" -nostdinc++ -isystem "%s/usr/include/c++/v1" %s "$@" '
                    "-nostdlib++ -lc++_static -lc++abi -lunwind --target=aarch64-linux-android29\n"
                    "else\n"
                    '  exec "%s/clang++" --sysroot="%s" -nostdinc++ -isystem "%s/usr/include/c++/v1" "$@" '
                    "--target=aarch64-linux-android29\n"
                    "fi\n"
                ) % (bin_dir, sysroot, sysroot, link_flags, bin_dir, sysroot, sysroot)
            else:
                text = (
                    "#!/data/data/com.termux/files/usr/bin/sh\n"
                    "link=1\n"
                    'for a in "$@"; do\n'
                    '  case "$a" in -c|-E|-S|-M|-MM|-MD|-MMD|-###|-v|--version|-dumpversion) link=0 ;; esac\n'
                    "done\n"
                    'if [ "$link" = 1 ]; then\n'
                    '  exec "%s/clang" --sysroot="%s" %s "$@" --target=aarch64-linux-android29\n'
                    "else\n"
                    '  exec "%s/clang" --sysroot="%s" "$@" --target=aarch64-linux-android29\n'
                    "fi\n"
                ) % (bin_dir, sysroot, link_flags, bin_dir, sysroot)
        elif name in NATIVE_TOOLS:
            text = '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s/%s" "$@"\n' % (
                bin_dir,
                NATIVE_TOOLS[name],
            )
        else:
            text = '#!/data/data/com.termux/files/usr/bin/sh\nexec "%s" "%s" "$@"\n' % (
                box,
                real_bin / name,
            )
        path.write_text(text, encoding="utf-8")
        path.chmod(0o755)

    toolchain = compat / "build/cmake/android.toolchain.cmake"
    text = toolchain.read_text(encoding="utf-8")
    anchor = "endif()\n\n# Toolchain.\nset(ANDROID_TOOLCHAIN_ROOT"
    if anchor not in text:
        raise SystemExit("unexpected NDK toolchain layout: %s" % toolchain)
    text = text.replace(
        anchor,
        "endif()\n\n# Termux aarch64 host: force the x86_64 host tag used by the shim.\n"
        "if(NOT ANDROID_HOST_TAG)\n  set(ANDROID_HOST_TAG linux-x86_64)\nendif()\n\n"
        "# Toolchain.\nset(ANDROID_TOOLCHAIN_ROOT",
        1,
    )
    sysroot_line = 'set(CMAKE_SYSROOT "${ANDROID_TOOLCHAIN_ROOT}/sysroot")'
    if sysroot_line not in text:
        raise SystemExit("unexpected NDK toolchain layout: %s" % toolchain)
    text = text.replace(
        sysroot_line,
        sysroot_line
        + '\nset(CMAKE_C_FLAGS_INIT "${CMAKE_C_FLAGS_INIT} -B${ANDROID_TOOLCHAIN_ROOT}/bin")'
        + '\nset(CMAKE_CXX_FLAGS_INIT "${CMAKE_CXX_FLAGS_INIT} -B${ANDROID_TOOLCHAIN_ROOT}/bin")',
        1,
    )
    toolchain.write_text(text, encoding="utf-8")
    return compat


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--sdk",
        default=os.environ.get("ANDROID_SDK_ROOT")
        or os.environ.get("ANDROID_HOME")
        or str(Path.home() / "android-sdk"),
        help="Android SDK root (default: $ANDROID_SDK_ROOT or ~/android-sdk)",
    )
    parser.add_argument("--ndk-version", default=DEFAULT_NDK_VERSION)
    parser.add_argument(
        "--out",
        default=str(Path.home() / ".cache/better-endfield-next/termux-ndk"),
        help="output directory",
    )
    parser.add_argument(
        "--box64",
        default="",
        help="box64 binary (default: $PREFIX/glibc/bin/box64 or $PREFIX/bin/box64)",
    )
    args = parser.parse_args()

    sdk = Path(args.sdk)
    ndk = sdk / "ndk" / args.ndk_version
    if not ndk.is_dir():
        raise SystemExit("NDK %s not found under %s" % (args.ndk_version, sdk))
    prefix = termux_prefix()
    box64 = Path(args.box64) if args.box64 else find_box64()
    out_root = Path(args.out)
    out_root.mkdir(parents=True, exist_ok=True)

    write_box64_wrapper(out_root, box64)
    compat = make_compat_ndk(out_root, ndk, prefix)
    shim = write_cmake_shim(out_root, sdk, prefix)
    print("compat NDK :", compat)
    print("cmake shim :", shim)
    print(
        "\nexport BE_TERMUX_NDK=%s\n"
        "printf 'sdk.dir=%s\\ncmake.dir=%s\\n' > android/local.properties\n" % (
            compat,
            sdk,
            shim.parent,
        )
    )


if __name__ == "__main__":
    main()
