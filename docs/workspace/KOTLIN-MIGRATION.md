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
| 0 | Android 构建接入 Kotlin 插件（`kotlin-android`），版本与上游 AGP/Java 17 对齐 | `assembleRelease` 不变、APK 正常 |
| 1 | 迁移本分支新增、上游不动的文件：`BemSource`、`TextureSupport`、`BemConverter`、`DiagnosticsExport/Provider` | 同一实现、lint/编译通过 |
| 2 | 迁移低冲突模块：`Text`、`AppLanguage`、`AboutPage`、`BemImportRequest/BemImportStream` | 导入流程 + 设置页冒烟 |
| 3 | 迁移导入核心：`BemInstaller`、`BemInstallPage`、`BemImportArchive`（与上游协商/改为 Kotlin 后同步 upstream 差异） | 多格式导入、纹理重写、真机 |
| 4 | 迁移 UI 壳与 Activity/Provider，更新 Manifest | 启动、LSPosed 注入、更新检查 |
| 5 | 删除 `java/` 下空壳，构建仅 Kotlin + native | CI 出包 |

## 防回退

- 阶段 2 起，每次上游同步后跑一次迁移后模块的冒烟；出现回归先回退该模块 Kotlin 文件；
- Kotlin 版本、JVM target 由 `android/build.gradle.kts` 统一，禁止各模块单独设定；
- 不在迁移提交里混入功能变更。
