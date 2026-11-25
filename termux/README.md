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

## arm64 Termux 直接跑 Gradle 的已知限制（2026-11-25 实测）

在 Termux（aarch64）本机直接 `./gradlew :app:assembleDebug` 时，Gradle/Java 侧能跑通，
但 Android SDK/AGP 默认带的是 **x86_64 宿主机工具**，需要逐个替换：

| 阻塞点 | 现象 | 处理 |
|---|---|---|
| Gradle 9.3.1 下载 | `downloads.gradle.org` 在手机上超时 | 临时把 wrapper `distributionUrl` 指向镜像（如 `mirrors.cloud.tencent.com/gradle/`）；CI 仍用官方源 |
| core 资源缺失 | `resources/generated` 为空，`BuildResourcePacks.py` 无输入 | 从 releases 的 `game-resources` 取 `BE-Core-Android-<ver>.zip`，按 `scripts/ci/fetch-core.ps1` 的映射还原 `android/resources/{character-presets,voice-catalog-index}.json` |
| SDK 的 CMake/Ninja | `cmake: 1: Syntax error: ")" unexpected`（ELF x86-64 无法执行） | `android/local.properties` 写 `cmake.dir=/data/data/com.termux/files/usr`，并临时去掉 `externalNativeBuild.cmake.version`（AGP 才会用自定义 cmake） |
| AGP 下载的 aapt2 | 同上，`aapt2 Daemon startup failed` | `-Pandroid.aapt2FromMavenOverride=/data/data/com.termux/files/usr/bin/aapt2`（Termux 的 arm64 aapt2） |
| **NDK 工具链** | `CMAKE_C_COMPILER .../prebuilt/bin/clang is not a full path to an existing compiler tool` | **硬阻塞**：NDK 只带 `linux-x86_64/darwin-x86_64/windows-x86_64` 宿主工具，arm64 Termux 无法执行 |

结论：本机能完成 **Gradle 配置、core 资源打包、资源合并、aapt2 资源链接**（实测都过了），
卡在 `configureCMake`。可选路径：

1. **官方路径**：在 GitHub Actions（`.github/workflows/android-release.yml`）构建，或 x86_64 主机；
2. **本机绕过**：用 Termux clang + NDK sysroot 写自定义 CMake toolchain，预编译
   `libbetterendfieldnext_android.so` 放进 `jniLibs/arm64-v8a`，Gradle 只负责打包；
3. **x86_64 proot/QEMU**：慢，但能完全复用官方 NDK 工具链。

复现命令（本机，含上述替换）：

```bash
printf 'sdk.dir=%s\ncmake.dir=%s\n' "$HOME/android-sdk" "$PREFIX" > android/local.properties
./gradlew :app:assembleDebug \
  -PworkspaceConfig="$PWD/ci-workspace.json" \
  -Pandroid.aapt2FromMavenOverride="$PREFIX/bin/aapt2"
```

## 本机构建（aarch64 Termux，opt-in）

上面的替换项已经固化成一条可重复路径，**CI / x86_64 主机不受影响**（靠环境变量
`BE_TERMUX_NDK` 启用）。2026-11-25 实测产出可安装的 debug APK。

### 准备

```bash
# 1. 安装与项目 pin 对齐的 NDK（27.2.12479018）
sdkmanager --install "ndk;27.2.12479018"
# 2. 需要 box64（少数无 arm64 版本的 NDK 工具走它）
pkg install box64-glibc      # 或已有的 box64
# 3. 生成 shim（NDK 数据 + Termux clang/llvm-* 原生工具 + box64 兜底）
python termux/setup-termux-ndk.py
# 4. 让 AGP 用 shim 的 cmake/ninja（local.properties 已被 gitignore）
printf 'sdk.dir=%s\ncmake.dir=%s\n' "$HOME/android-sdk" \
  "$HOME/.cache/better-endfield-next/termux-ndk/android-cmake" > android/local.properties
```

### 构建

```bash
export BE_TERMUX_NDK="$HOME/.cache/better-endfield-next/termux-ndk/ndk-compat"
cd android
./gradlew :app:assembleDebug \
  -PworkspaceConfig="$PWD/../ci-workspace.json" \
  -Pandroid.aapt2FromMavenOverride="$PREFIX/bin/aapt2" \
  --offline --build-cache --console=plain
```

`ci-workspace.json` 只需 `{"schema_version":1,"tools":{"android_sdk":"<SDK>"}}`。

**注意**：`BE_TERMUX_NDK` 是 Gradle **daemon 启动时**读的环境变量。改了它要先
`./gradlew --stop` 再构建，否则 daemon 还用旧环境。

### 实测数字（本机，aarch64）

| 场景 | 耗时 |
|---|---|
| 首次完整 `assembleDebug`（含 CMake 配置 + 150 个原生目标） | 约 2 分 31 秒 |
| 改一个 `.cpp` 后增量 | 8–10 秒（4 个任务执行） |

产物：`build/next/android/gradle/app/outputs/apk/debug/app-debug.apk`，
内含 `lib/arm64-v8a/libbetterendfieldnext_{android,installer}.so`（ELF aarch64，
`built by NDK r27c`）与 `assets/bundled-resources/core.zip`。

### 速度结论（参考 JMNeXt 的实测笔记）

- **别用 `--no-daemon`**：每次多 fork 一个 JVM，实测慢 7–16 秒；
- **常驻 daemon + `--offline` + `--build-cache`**，并尽量把多个任务合并进同一次调用；
- `org.gradle.workers.max=2` 反而更慢（热降频/内存先到头），不要设；
- **configuration cache 暂不可用**：`settings.gradle.kts` 在配置阶段起 Python
  (`workspace_config.py`) 且 Kotlin 任务序列化有问题，Gradle 会直接判失败——保持关闭；
- release 的耗时大头通常是 R8 与 `lintVital*`，日常迭代用 `assembleDebug`；需要时再单独跑 lint。

### 本机构建的实现要点（都 gated 在 `BE_TERMUX_NDK` 下）

- `ndkPath` 指向 shim；`cmake.version` 让位给 `cmake.dir`（自建 shim）；
- shim 里 `clang`/`clang++` 是 **arm64 Termux clang**，编译/链接分别处理静态 libc++，
  其余 `llvm-*`、`ld.lld` 也用 Termux 版本；仅个别无替代的工具回退 box64；
- CMake 配置仍由 box64 跑 x86_64 版本（一次性，约 2 分钟）；
- `-DCMAKE_SUPPRESS_REGENERATION=ON` + 配置后把生成的 ninja 规则里的 x86_64
  cmake 路径改写成 shim，避免 `ninja` 直接 exec x86_64 二进制。

### Release（本机）

```bash
export BE_TERMUX_NDK="$HOME/.cache/better-endfield-next/termux-ndk/ndk-compat"
cd android
./gradlew :app:assembleRelease \
  -PworkspaceConfig="$PWD/../ci-workspace.json" \
  -Pandroid.aapt2FromMavenOverride="$PREFIX/bin/aapt2" \
  --offline --build-cache --console=plain
```

- 本机路径（`BE_TERMUX_NDK` 存在时）自动关闭 `lintVitalRelease`：
  它需要额外 Maven 依赖、且是 release 墙钟时间大头；CI 不受影响。
- 首次 release 含原生 Release 重编译 + R8，实测约 **6 分钟**；之后增量约 **25 秒**。
- 没有签名配置时产物是未签名 APK：
  `build/next/android/gradle/app/outputs/apk/release/app-release-unsigned.apk`，
  同时归档到 `releases/<version>/BetterEndfieldNext-<version>-Android-arm64.apk`。
- R8 混淆映射：`build/next/android/gradle/app/outputs/mapping/release/mapping.txt`。
- 本机测试可用调试密钥签名（密钥只放 `build/`，不入库）：

```bash
keytool -genkeypair -keystore build/check/termux-debug.keystore -alias androiddebugkey \
  -storepass android -keypass android -keyalg RSA -keysize 2048 -validity 10000 \
  -dname "CN=Android Debug,O=Android,C=US"
apksigner sign --ks build/check/termux-debug.keystore --ks-pass pass:android \
  --key-pass pass:android --out build/check/app-termux-signed.apk <unsigned.apk>
```

> 注意：镜像的 Gradle wrapper 只用于本机（`downloads.gradle.org` 在部分网络不可达）；
> 仓库里的 `distributionUrl` 保持官方地址，本地临时改镜像即可。
