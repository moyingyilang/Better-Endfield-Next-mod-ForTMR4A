#!/data/data/com.termux/files/usr/bin/bash
# SPDX-License-Identifier: MPL-2.0
# 验收 APK 的“产物形态”：包名、versionCode/versionName、签名。
# 判据来自目录而不是命令退出码；aapt2 是 glibc 二进制，在 Termux 宿主会
# Exec format error，所以优先放进容器跑。
# 改编自 android-termux-builders4JMNeXt（MPL-2.0）。
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
[ -f "$HERE/build.env" ] && . "$HERE/build.env" || { echo "缺 termux/build.env"; exit 1; }
: "${PROJECT_DIR:?}" "${ANDROID_SDK:?}" "${AAPT2_BUILD_TOOLS:?}"

APK_DIR="$PROJECT_DIR/build/next/android/gradle/app/outputs/apk/release"
PACKAGE="dev.betterendfield.next"
VERSION="$(sed -n 's:.*<Version>\(.*\)</Version>.*:\1:p' "$PROJECT_DIR/Directory.Build.props" | head -1 | tr -d '[:space:]')"
[ -n "$VERSION" ] || { echo "读不到 Directory.Build.props 的 Version"; exit 1; }
IFS=. read -r major minor patch <<EOF
$VERSION
EOF
EXPECTED_CODE=$((major * 10000 + minor * 100 + patch))
echo "期望：package=$PACKAGE versionName=$VERSION versionCode=$EXPECTED_CODE"

AAPT2="$ANDROID_SDK/build-tools/$AAPT2_BUILD_TOOLS/aapt2"
APKSIGNER="$ANDROID_SDK/build-tools/$AAPT2_BUILD_TOOLS/apksigner"
run_aapt2() {
  if [ -n "${CHROOT_ROOT:-}" ]; then
    su -c "chroot '$CHROOT_ROOT' /usr/bin/env -i PATH=/usr/bin:/bin /bin/bash -c '$AAPT2 dump badging \"$1\" 2>&1 | head -3'"
  else
    "$AAPT2" dump badging "$1" 2>&1 | head -3
  fi
}

shopt -s nullglob
apks=("$APK_DIR"/*.apk)
[ "${#apks[@]}" -gt 0 ] || { echo "没有 APK：$APK_DIR"; exit 1; }
rc=0
for f in "${apks[@]}"; do
  echo "=== $(basename "$f") ==="
  badging="$(run_aapt2 "$f")"
  echo "$badging" | sed 's/^/  /'
  echo "$badging" | grep -q "package: name='$PACKAGE'" || { echo "  包名不符"; rc=1; }
  echo "$badging" | grep -q "versionCode='$EXPECTED_CODE'" || { echo "  versionCode 不符"; rc=1; }
  echo "$badging" | grep -q "versionName='$VERSION'" || { echo "  versionName 不符"; rc=1; }
  if [ -n "${CHROOT_ROOT:-}" ]; then
    su -c "chroot '$CHROOT_ROOT' /usr/bin/env -i PATH=/usr/bin:/bin /bin/bash -c '$APKSIGNER verify \"$f\" >/dev/null 2>&1'" \
      && echo "  apksigner: 通过" || { echo "  apksigner: 失败（未签名或签名无效）"; rc=1; }
  elif [ -x "$APKSIGNER" ]; then
    "$APKSIGNER" verify "$f" >/dev/null 2>&1 && echo "  apksigner: 通过" || { echo "  apksigner: 失败"; rc=1; }
  else
    echo "  apksigner: 不可用（跳过；发布前必须在签名环境复验）"
  fi
done
[ "$rc" = 0 ] && echo "APK 验收通过" || echo "APK 验收失败"
exit "$rc"
