#!/data/data/com.termux/files/usr/bin/bash
# SPDX-License-Identifier: MPL-2.0
# 从 Termux 进入构建环境。
# 改编自 android-termux-builders4JMNeXt（https://github.com/moyingyilang/JMNeXt），
# 仅保留本项目需要的挂载自检与容器可见性检查。
#
# 两种模式：
#   CHROOT_ROOT 非空 -> 在 chroot/proot 容器里构建（推荐，glibc 工具链完整）
#   CHROOT_ROOT 为空 -> 直接在 Termux 宿主构建（需宿主已有 JDK/SDK/NDK/Gradle）
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
[ -f "$HERE/build.env" ] && . "$HERE/build.env" || { echo "缺 termux/build.env（可从 env.example 复制）"; exit 1; }
: "${PROJECT_DIR:?PROJECT_DIR 未设置}"
[ -d "$PROJECT_DIR" ] || { echo "找不到项目：$PROJECT_DIR"; exit 1; }

if [ -z "${CHROOT_ROOT:-}" ]; then
  echo "模式：Termux 宿主直接构建"
  for p in "${ANDROID_SDK:?ANDROID_SDK 未设置}"; do :; done
  [ -d "$ANDROID_SDK" ] || { echo "找不到 Android SDK：$ANDROID_SDK"; exit 1; }
  echo "  项目：$PROJECT_DIR"
  echo "  SDK ：$ANDROID_SDK"
  exit 0
fi

echo "模式：容器构建（$CHROOT_ROOT）"
[ -d "$CHROOT_ROOT" ] || { echo "找不到容器根：$CHROOT_ROOT"; exit 1; }

# /dev 必须是可用设备节点；rootfs 在 /data 上时 SELinux 不允许直接 mknod。
if ! su -c "chroot '$CHROOT_ROOT' /bin/sh -c 'echo probe > /dev/null'" 2>/dev/null; then
  echo "  重建容器 /dev"
  su -c "
    mountpoint -q '$CHROOT_ROOT/dev' || mount -t tmpfs tmpfs '$CHROOT_ROOT/dev'
    for n in 'null c 1 3' 'zero c 1 5' 'random c 1 8' 'urandom c 1 9'; do
      set -- \$n
      [ -e '$CHROOT_ROOT/dev/'\$1 ] || mknod -m 666 '$CHROOT_ROOT/dev/'\$1 \$2 \$3 \$4
    done
  " || { echo "  /dev 重建失败"; exit 1; }
fi

# /proc、/sys 缺失会让 JVM 起不来，且错误不明显。
for m in proc sys; do
  mp="$CHROOT_ROOT/$m"
  mountpoint -q "$mp" && continue
  if [ "$m" = proc ]; then mount -t proc proc "$mp"; else mount -o bind /sys "$mp"; fi \
    || echo "  挂载 $mp 失败"
done

# 绑定挂载项目与 SDK（按 build.env 的 MOUNT_DIRS）。
need=0
for d in ${MOUNT_DIRS:-}; do
  rel="${d#"$HOME"/}"
  target="$CHROOT_ROOT$d"
  [ -d "$d" ] || continue
  if ! su -c "ls -d '$target' >/dev/null 2>&1"; then
    su -c "mkdir -p '$target' && mount -o bind '$d' '$target'" \
      && echo "  已挂载 $d" || { echo "  挂载失败：$d"; need=1; }
  fi
done
[ "$need" = 0 ] || { echo "有挂载失败，先修好再构建"; exit 1; }

su -c "chroot '$CHROOT_ROOT' /usr/bin/env -i PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin HOME=/root /bin/bash -c '
  ls -d \"$PROJECT_DIR\" >/dev/null 2>&1 && echo \"  容器内可见项目\" || { echo \"  容器内看不到项目（绑定挂载缺失）\"; exit 1; }
  ls -d \"$ANDROID_SDK/platforms\" >/dev/null 2>&1 && echo \"  容器内可见 Android SDK\" || { echo \"  容器内看不到 Android SDK\"; exit 1; }
'"
