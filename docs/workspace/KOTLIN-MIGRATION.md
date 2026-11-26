# Java → Kotlin 迁移计划

上游 Android 端是 Java（约 1.5 万行）。本分支目标：**新代码 Kotlin 优先，存量逐步迁移**，
且不破坏“跟随上游”的同步节奏。

## 原则

1. **上游仍在频繁改的文件不迁**：每次上游同步都会冲突，迁移收益被冲突吃掉；
2. **新文件一律 Kotlin**：新增 Android 逻辑（如导入回退、纹理能力、诊断导出）用 Kotlin；
3. **按模块整包迁移**：一个文件迁完即删旧 Java，保持同一提交内完成，可回退；
4. **行为对拍**：迁移前后跑同一套构建 + 导入冒烟；涉及 native 边界的（JNI 回调）先确认签名不变；
5. **不迁的平台壳**：`FrameworkBootstrapActivity`、`Provider`、`Receiver` 这类 AndroidManifest 组件
   需要严格包名/类名，迁移时同步改 Manifest，且必须真机验证。

## 阶段

| 阶段 | 内容 | 验收 |
|---|---|---|
| 0（已完成 2026-11-26） | **AGP 9 内置 Kotlin 2.2.10**（不再应用 `kotlin-android`），只加 `org.jetbrains.kotlin.plugin.compose` 2.2.10；`buildFeatures.compose=true` | `assembleRelease` 通过，APK 8.2 MB |
| 1（进行中） | 已完成：Compose 主题（`ui/theme/EndfieldTheme.kt`，基于 `ui-kit` token）+ 模型页试点 `ui/pilot/ImportPilotActivity.kt`（导入入口、已安装列表、Accent 配色），`AboutPage` 增加「新版模型页面」入口；待迁：`BemSource`、`TextureSupport`、`DiagnosticsExport/Provider` → Kotlin | 试点页编译/装机可见，导入按钮可拉起原流程 |
| 2 | 迁移低冲突模块：`Text`、`AppLanguage`、`AboutPage`、`BemImportRequest/BemImportStream` | 导入流程 + 设置页冒烟 |
| 3 | 迁移导入核心：`BemInstaller`、`BemInstallPage`、`BemImportArchive`（与上游协商/改为 Kotlin 后同步 upstream 差异） | 多格式导入、纹理重写、真机 |
| 4 | 迁移 UI 壳与 Activity/Provider，更新 Manifest | 启动、LSPosed 注入、更新检查 |
| 5 | 删除 `java/` 下空壳，构建仅 Kotlin + native | CI 出包 |

## 防回退

- 阶段 2 起，每次上游同步后跑一次迁移后模块的冒烟；出现回归先回退该模块 Kotlin 文件；
- Kotlin 版本、JVM target 由 `android/build.gradle.kts` 统一，禁止各模块单独设定；
- 不在迁移提交里混入功能变更。

## 设计系统层（2026-11-26 完成）

- 三套设计系统：`Endfield UI X`（token 生成 Material3 配色）、`Material Design`（Material3 + 动态取色）、`MIUIX`；
- 统一语义组件：`ui/design/DesignComponents.kt`（`DesignScreen/DesignCard/DesignText/DesignButton/DesignRow`），页面只写一次，三套系统各自渲染；
- 选择持久化：`uidesign` 设置（key `ui.design_system`），试点页内置切换器；
- **MIUIX 依赖固定 `top.yukonga.miuix.kmp:miuix-android:0.7.2`**：AGP 9.1 内置 Kotlin 2.2.10 只能读 2.2.x metadata，0.8+ 用 Kotlin 2.3/2.4 编译会拒绝；升级 AGP 之前不要升 Miuix；
- 新增 UI 一律 Kotlin + 语义组件；旧 Java/XML 页替换时只改 `BemInstallPage` 等入口，不动导入逻辑。
