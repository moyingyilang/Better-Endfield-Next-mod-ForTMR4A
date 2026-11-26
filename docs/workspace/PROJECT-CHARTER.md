# 项目章程（Better Endfield Next mod ForTMR4A）

## 定位

这是 **Better Endfield Next（BEN）的 mod 分支**，不是另起炉灶的替代品：

1. **跟随官方更新**：持续 merge `Dr-hydra/Better-Endfield` 的 `main`，保留上游全部功能与资源版本；
   上游修的东西（纹理设备能力、资源 1.7.2、相机、音乐、mod-center…）我们都要有；
2. **在官方之上加功能**：ForTMR / ForTMR2 集成、统一多格式导入（BEM / EFMI / EFF / RAR/7z/tar…）、
   CRX 原生链（BEM 读写、BEM→IR、协议转换）；
3. **重写我们不喜欢的部分**：只重写实现，不改上游行为——
   - Python → CRX（C11/C17 主链路、Rust 叶子、shell/Rust 构建胶水）；
   - Java → Kotlin（Android UI 逐步迁移，新代码 Kotlin 优先）；
   - Windows 侧代码归档到 `legacy/windows/` 保留可追溯，不在本分支维护。

## 分支约定

| 项 | 约定 |
|---|---|
| 上游 | `upstream` remote = `Dr-hydra/Better-Endfield`；同步流程见 `UPSTREAM-SYNC.md` |
| 目标平台 | Android（arm64-v8a）；Linux 仅作构建宿主 |
| 语言 | C11/C17 主链路，Rust 叶子（C ABI，panic=abort），C++17/20 宿主；**Android UI 全面 Kotlin（Compose）** |
| UI | Kotlin + Compose；三套设计系统：**Endfield UI X / Material Design / MIUIX**，用户可切换 |
| 禁止 | Python / JS 实现业务逻辑；C# 只读（`legacy/`）；Android UI 新增 Java |
| 第三方 | 内置库需在 `CREDITS.md` / `VENDORED.md` 记录来源与许可 |
| 验收 | `assembleRelease` 出包 + `efml validate` + 真机导入/加载冒烟 |

## 技术栈迁移原则

- **只重写实现，不重写行为**：迁移前后用同一份输入/输出做对拍（BEM、efml validate、真机）；
- **先测量再迁移**：按台账（`CRX-REWRITE.md`）逐段推进，优先高收益、低耦合模块；
- **上游还在改的文件不要同时大改**：先合上游，再在其上做小步迁移，避免每次同步冲突；
- **新文件遵守新栈**：新增 Android 代码用 Kotlin，新增解析/转换逻辑用 C/Rust，不引入新的 Python/JS；
- **迁移必须可回退**：一个模块迁完、验收通过、上游同步一次无冲突后才删旧实现。

## 当前状态（2026-11-26）

- 已同步上游 **4.2.5**（merge commit，含 4.2.2 纹理设备能力、资源 1.7.2、相机/音乐/mod-center）；
- 本分支特有：Android-only 结构、多格式导入、EFMI/EFF 本地转换、libarchive 解包、ForTMR2 静态链接；
- 技术栈迁移：CRX 主链路 6/10；Java→Kotlin 尚未开始（见 `KOTLIN-MIGRATION.md`）。
