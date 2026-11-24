# termux/ —— ForTMR4A 的 Termux 构建链

在 Termux 上构建本分支（Android-only）的一套脚本。**改编自**
[android-termux-builders4JMNeXt](https://github.com/moyingyilang/JMNeXt)（MPL-2.0）：
保留其“进容器 → 出包 → 以产物形态验收”的结构，只把任务与判据换成本项目。

> 许可：本目录脚本沿用上游的 **MPL-2.0**（文件级传染）；项目其余部分为 AGPL-3.0。

## 为什么不用 `./gradlew` 直接跑

不是不能用，而是 Termux 上有几个反复踩的坑，脚本把它们固化成检查：

1. 容器/绑定挂载掉线时，报错是“目录不存在”，容易误判成项目丢了；
2. `debug` 不跑 R8，跨模块重复类/缺类/签名问题查不出来 → 必须 `assembleRelease`；
3. 构建失败时旧 APK 还在，`ls` 有文件≠本次构建成功 → 必须比对产物时间戳；
4. `aapt2` 是 glibc 二进制，在 Termux 宿主上跑会 `Exec format error` → 进容器验包；
5. `apksigner` 是脚本 + jar，Termux 侧可跑，但签名密钥不许入库。

## 用法

```bash
cp termux/env.example termux/build.env   # build.env 已被 .gitignore
$EDITOR termux/build.env                 # 填 PROJECT_DIR / CHROOT_ROOT / ANDROID_SDK

./termux/10-enter.sh        # 环境自检（容器模式会补挂载并验证可见性）
./termux/20-build-android.sh # assembleRelease + 产物新鲜度自检
./termux/30-verify-apk.sh    # 包名 / versionCode / versionName / 签名
```

两种模式（由 `CHROOT_ROOT` 是否为空决定）：

| 模式 | 适用 | 要求 |
|---|---|---|
| 容器（推荐） | Termux + chroot/proot rootfs | root/su、容器内有 JDK 21 + Android SDK/NDK + Gradle |
| 宿主 | Termux 本机已装 JDK/SDK/NDK/Gradle | 无需 root；`aapt2` 需 arm64 版本 |

产物路径（Gradle buildDir 被 `android/workspace.gradle.kts` 重定向）：

```
build/next/android/gradle/app/outputs/apk/release/*.apk
```

验收判据来自 `Directory.Build.props`：`package=dev.betterendfield.next`，
`versionCode = major*10000 + minor*100 + patch`，`versionName = <Version>`。

## 与 CI 的关系

`.github/workflows/android-release.yml` 是 CI 侧的等价流程（ubuntu + JDK 21 + NDK）；
Termux 链是本地权威构建，两者都以 `Directory.Build.props` 为版本唯一真源。
