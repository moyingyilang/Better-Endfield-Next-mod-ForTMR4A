#!/data/data/com.termux/files/usr/bin/bash
# SPDX-License-Identifier: MPL-2.0
# 打 Android release 包，并做“产物新鲜度”自检：
# 构建失败时旧 APK 还在原地，只看 ls 会误判成功，所以必须比对时间戳。
# 改编自 android-termux-builders4JMNeXt（MPL-2.0）。
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
[ -f "$HERE/build.env" ] && . "$HERE/build.env" || { echo "缺 termux/build.env"; exit 1; }
: "${PROJECT_DIR:?}" "${ANDROID_RELEASE_TASKS:?}"
"$HERE/10-enter.sh"

# 本项目 Gradle 的 buildDir 被 workspace.gradle.kts 重定向到 <repo>/build/next/android/gradle/app。
APK_DIR="$PROJECT_DIR/build/next/android/gradle/app/outputs/apk/release"
rm -f "$APK_DIR"/*.apk 2>/dev/null || true
start="$(date +%s)"

echo "=== 构建：$ANDROID_RELEASE_TASKS ==="
if [ -n "${CHROOT_ROOT:-}" ]; then
  su -c "chroot '$CHROOT_ROOT' /usr/bin/env -i PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin HOME=/root JAVA_HOME='$CONTAINER_JDK' LD_LIBRARY_PATH='$CONTAINER_JDK/lib' TERM=xterm /bin/bash -c '
    cd \"$PROJECT_DIR/android\" && chmod +x gradlew && nice -n 19 ./gradlew --console=plain --no-daemon $ANDROID_RELEASE_TASKS 2>&1 | tail -40
    pkill -f \"[K]otlinCompileDaemon\" 2>/dev/null || true
  '"
else
  ( cd "$PROJECT_DIR/android" && chmod +x gradlew && ./gradlew --console=plain --no-daemon $ANDROID_RELEASE_TASKS )
fi

echo "=== 产物新鲜度自检 ==="
shopt -s nullglob
apks=("$APK_DIR"/*.apk)
[ "${#apks[@]}" -gt 0 ] || { echo "  没有 APK：$APK_DIR"; exit 1; }
bad=0
for f in "${apks[@]}"; do
  printf "  %-42s %10s 字节  %s\n" "$(basename "$f")" "$(stat -c %s "$f")" "$(date -r "$f" '+%Y-%m-%d %H:%M')"
  [ "$(date -r "$f" +%s)" -ge "$start" ] || { echo "    产物早于本次构建，构建可能没真正跑成"; bad=1; }
done
[ "$bad" = 0 ] || exit 1
sha256sum "${apks[@]}"
echo "构建通过；接着用 30-verify-apk.sh 验包名/版本/签名"
