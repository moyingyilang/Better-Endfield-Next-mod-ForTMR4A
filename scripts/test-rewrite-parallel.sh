#!/data/data/com.termux/files/usr/bin/sh
# Host-side regression for the parallel texture rewrite path (no device, no Python).
#   scripts/test-rewrite-parallel.sh [input.bem]
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IN="${1:-$ROOT/build/check/effdiag/lvtsfw_eff_diag.bem}"
OUT="${BE_TEST_OUT:-$ROOT/build/check/rewrite-parallel}"
BIN="$OUT/harness"
mkdir -p "$OUT"
[ -f "$IN" ] || { echo "missing input: $IN" >&2; exit 1; }
clang++ -std=c++20 -O2 \
    -I"$ROOT/native/modules/custom_model" \
    -I"$ROOT/native/shared/third_party" \
    -I"$ROOT/native/shared/third_party/zstd/lib" \
    -I"$ROOT/native/shared/third_party/miniz" \
    "$ROOT/native/modules/custom_model/tests/rewrite_parallel_harness.cpp" \
    "$ROOT/native/modules/custom_model/bem.cpp" \
    "$ROOT"/native/shared/third_party/zstd/lib/common/*.c \
    "$ROOT"/native/shared/third_party/zstd/lib/decompress/*.c \
    "$ROOT"/native/shared/third_party/zstd/lib/compress/*.c \
    "$ROOT/native/shared/third_party/miniz/miniz.c" \
    -o "$BIN" -lpthread -lm
"$BIN" "$IN" "$OUT/out1.bem"
"$BIN" "$IN" "$OUT/out2.bem"
sha1=$(sha256sum "$OUT/out1.bem" | cut -d' ' -f1)
sha2=$(sha256sum "$OUT/out2.bem" | cut -d' ' -f1)
echo "out1 sha256=$sha1"
echo "out2 sha256=$sha2"
[ "$sha1" = "$sha2" ] && echo "deterministic: OK" || { echo "deterministic: FAIL" >&2; exit 1; }
BEM_CLI="$ROOT/native/bem/target/release/be-bem"
if [ -x "$BEM_CLI" ]; then
    "$BEM_CLI" validate "$OUT/out1.bem" || { echo "BEM validation failed" >&2; exit 1; }
fi
echo "done: $OUT"
