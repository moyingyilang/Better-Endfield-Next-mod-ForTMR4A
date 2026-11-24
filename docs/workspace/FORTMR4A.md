# ForTMR4A 分支：平台收敛与 ForTMR2 集成方案

> 状态：方案定稿，P0 部分已执行
> 日期：2026-11-24
> 分支：`better-endfield-next-mod-ForTMR4A`（Git 分支名不允许空格）
> 目录：`Better-Endfield-Next-mod-ForTMR4A`
> 回滚点：tag `pre-fortmr4a`（= 原 `main` / `6ad7719`）

## 1. 定位

本分支是 **ForTMR4A**：只面向《明日方舟：终末地》**Android** 端的 mod 层与运行时，是
[ForTMR2](https://github.com/moyingyilang/ForTMR2) 公开基础设施的下游消费者。
Linux 仅作为开发与构建宿主（Termux / 发行版），**不产出 Linux 运行时**；**Windows 支持移除**。

## 2. 平台矩阵

| 平台 | 角色 | 产物 | 说明 |
|---|---|---|---|
| Android 10+ / arm64 | **唯一产品平台** | APK（LSPosed API 102 模块）+ native `.so` | 注入、模块、游戏内面板 |
| Linux（Termux / 发行版） | 开发与构建宿主 | 仅构建中间物 | Gradle + NDK；不发布 Linux 二进制 |
| Windows | 已移除 | 无 | 源码暂留参考，不参与构建 |

## 3. 命名决定

| 项 | 取值 | 说明 |
|---|---|---|
| 显示名 | **Better Endfield Next mod ForTMR4A** | 按业主指定 |
| 仓库目录 | `Better-Endfield-Next-mod-ForTMR4A` | 文件系统友好 |
| Git 分支 | `better-endfield-next-mod-ForTMR4A` | `git check-ref-format` 通过；空格非法 |
| Android 包名 / 模块 ID | **保持不变**（`dev.betterendfield.next`、`betterendfieldnext_*`） | 发布身份与升级路径不可随意改 |
| C++ 标识符 / ABI | 不变 | `BE_ModuleApiV1` 等已冻结 |

**“要不要再改”的结论**：还需要再改三处，但都**不是**标识符层面：

1. **构建入口**：`scripts/*.ps1` + Windows CMake 入口 → Gradle/CMake（Android）+ Makefile/shell（Linux 宿主）。
2. **CI**：`.github/workflows/release.yml` 目前是 Windows 四段流水线 → 收敛为 Android 单产物。
3. **仓库清理**：`ui/`、`installer/`、`native/loaders/`、Windows 专用模块 → `legacy/windows/`（保留引用）或删除。

不建议改名的项：包名、模块 ID、`BetterEndfieldNext` 前缀、`BEM` 格式版本号——这些已出现在发布物、
用户数据目录与 mod 包内，改名等于制造不兼容升级。

## 4. 现状盘点（自研代码，非空行）

| 目标 | 规模 | 处置 |
|---|---:|---|
| `android/`（Java + JNI + Gradle） | 23,090 | **保留，主战场** |
| `native/modules/`（与 Android 共用 TU） | ~78,000 | 保留；仅按 Android 编译 |
| `native/shared/`（C ABI / Hook / `android_compat`） | ~8,000 | 保留；去掉 Windows 分支 |
| `native/loaders/`（注入器 / xinput） | 1,533 | **移除**（Android 用 LSPosed） |
| `ui/`（C# + XAML，WinUI 3） | 25,338 | **移除**（死代码，且违反 CRX） |
| `installer/`（Inno Setup） | 447 | **移除**（APK 分发） |
| Windows 专用模块（combat_stats/gacha/music/voice） | ~16,000 | 移除或转 Android 实现 |
| `web/` + `resources/`（TS/JS） | 13,591 | **迁出或停用**（CRX 禁 JS 实现） |
| `tools/`（Python 逆向/资产管线） | 36,916 | 逐步替换为 Rust CLI（BEM 转换器独立仓） |
| `scripts/`（PowerShell/Python） | 8,475 | 重写为 Makefile/shell + Gradle 任务 |
| `docs/`、`config/`、`manifests/` | — | 保留，标注 Windows 章节为历史 |

## 5. ForTMR2 接口边界

| ForTMR2 组件 | 接口 | 本分支用法 |
|---|---|---|
| `native/crates/efml-core` | `mod.toml` 清单、后端无关 IR、`identify` 策略、包校验（`forbid(unsafe_code)`） | mod 包的唯一格式定义；BEN 打包器产出该格式 |
| `native/crates/efml-cli` | `efml` CLI：validate / inspect / pack | 构建期校验与发布前检查 |
| `lib/include/efml4a.h` | `efml4a_source`（dir / mem 后端，C ABI，LGPL-3.0） | Android 运行时读取 mod 资源；VFS 后端由私有组件在进程边界提供 |
| `vulkan-layer/` | Vulkan 隐式层（C++） | 渲染替换后端；与 `il2cpp-hook` 形成两条后端路径 |
| `mods/example-mod` | 参考包（合成数据） | 转换器回归的黄金样本 |

**不越界**：VFS、`.blc/.chk`、密钥、游戏资源解包留在私有仓；公开仓只回答“拿到资源之后怎么用”。

## 6. BEN 能力 → ForTMR2 映射

| BEN 现有能力 | ForTMR2 落点 | 备注 |
|---|---|---|
| BEM 包（zstd + zip + astc） | `mod.toml` + IR + `checksums.txt` | 需 BEM→IR 转换器 |
| 模型替换（IL2CPP hook） | `[[mesh.replace]]` + `backend = "il2cpp-hook"` | 已有 Android JNI 模块 |
| 贴图/材质替换 | `[[texture.replace]]` / `[[material.override]]` + `vulkan-layer` | 需 Vulkan 后端验证 |
| 骨骼/形态键 | `[[skeleton.map]]` / `[[blendshape]]` | IR 已定义 |
| 相机 / 语音 / 动作 / UI / 寻访 | **非 IR 能力**，保留为 Android 运行时扩展 | 建议写 `[ext.ben.*]` 段；`efml-core` 明确忽略未知字段，向前兼容 |
| 游戏资源读取 | 私有 VFS 后端实现 `efml4a_source` | 进程边界接入，公开构建不依赖 |

## 7. CRX 合规矩阵

CRX = **C 主链路 / Rust 叶子 / C++ 宿主**，禁止 Python / JavaScript 实现，C# 仅只读参考。

| 资产 | 现语言 | CRX 判定 | 处置 |
|---|---|---|---|
| Android JNI 运行时、Hook broker | C++ | 宿主，允许 | 保留（`extern "C"` 边界） |
| 资源读取/生命周期 | C | 主链路 | 由 ForTMR2 `lib/` 的 C 接口承载 |
| mod 清单/IR/校验 | Rust | 叶子 | 由 `efml-core` 承载 |
| BEM 格式解析/转换 | Python（+C++ 双实现） | 叶子 → Rust | 迁到 Rust crate（独立仓），C ABI/WASM 产物 |
| BEM 浏览器预览 | JS | 允许的宿主胶水 + WASM | 解码用 Rust→WASM，JS 只做壳；或迁出 |
| WinUI 桌面端 | C# + XAML | 禁止新增实现 | 移除 |
| 战斗分析网页 | TS/JS | 禁止实现 | 迁出独立仓或停用 |
| 构建/资源脚本 | PowerShell + Python | 仅允许 shell | 改 Makefile/shell；生成器换 Rust CLI |
| Android 应用壳 | Java | 平台约束 | 保留（Xposed 入口/Activity/UI） |

## 8. 目标架构

```
┌──────────────────────────── Android (arm64) ────────────────────────────┐
│  Java 壳（LSPosed API 102, dev.betterendfield.next）                      │
│    └─ JNI ─► C++ 宿主（Hook broker/Dobby、模块调度、android_compat）     │
│                  ├─ C 资源层：efml4a_source（ForTMR2 lib/）              │
│                  └─ Rust 叶子：mod 校验/IR 解析/编解码（efml-core 等）    │
├──────────────────────────── 构建宿主：Linux ─────────────────────────────┤
│  Gradle + NDK/CMake ─► APK                                               │
│  Rust CLI（BEM→IR 转换、pack、validate）─► mod 包（mod.toml + data）     │
└──────────────────────────────────────────────────────────────────────────┘
        ForTMR2 公开仓：efml-core / efml-cli / lib / vulkan-layer
        私有仓：VFS 后端、密钥、资源解包（进程边界）
```

## 9. 分阶段计划

| 阶段 | 内容 | 验收 | 预估 |
|---|---|---|---|
| **P0（已做一半）** | 分支/目录改名、README 平台声明、回滚 tag | 本次提交 | — |
| P0.5 | 移除 Windows 构建入口；CI 收敛为 Android；`scripts/` 改为 shell/Makefile | Linux 宿主上 `./gradlew :app:assembleRelease` 通过；CI 无 Windows job | 3–5 天 |
| P1 | Windows 资产清退：`ui/`、`installer/`、`native/loaders/`、Windows 专用模块迁 `legacy/windows/`（不删，保留许可与来源） | 构建只依赖 `android/` + 共用 native TU + `tools` | 1–2 周 |
| P2 | 接入 ForTMR2：submodule（pin commit）、CMake 消费 `lib/`、`efml-cli` 校验 mod、C ABI shim crate | `mods/example-mod` 能在 Android 侧解析/校验；`efml-cli validate` 通过 | 2–3 周 |
| P3 | BEM→IR 转换器（Rust 叶子，独立仓/子目录），替换 Python 转换链 | BEM 1.0–1.5 黄金样本逐字节对拍；`efml-cli` 可打包 | 3–4 周 |
| P4 | CRX 清退：`web/` 迁出、JS 预览换 WASM、Python 实现降级为构建 glue | 仓库无 Python/JS 实现代码 | 2–3 周 |
| P5 | 发布链：APK 签名 + mod 包分发；文档/索引更新 | 一次完整发布 | 1 周 |

总计：**P0–P2 约 1 个月可跑通主链路**，P3–P5 再 1.5–2 个月。

## 10. 风险与开放问题

| 风险 | 影响 | 缓解 |
|---|---|---|
| `efml-core` 是 `forbid(unsafe_code)` | 不能直接在里面写 C ABI | 单开 `efml-ffi` crate 做 `#[no_mangle] extern "C"` shim |
| 双后端（`il2cpp-hook` vs `vulkan-layer`）行为不同 | mod 表现不一致 | 先锁定 `il2cpp-hook`；Vulkan 仅作实验后端并标注 |
| BEM→IR 语义不一一对应 | 转换丢失（如 sRGB/Linear） | IR 缺少的字段先写 `[ext.ben.*]`，再向上游提格式扩展 |
| Windows 模块版权/许可（EIEM AGPL 快照） | 清理时不能丢来源 | `legacy/windows/` 保留 THIRD_PARTY_NOTICES 与 LICENSE |
| Python 36k 行迁移 | 工具链空窗期 | 先 Rust CLI 包住旧 Python（subprocess），再逐模块替换 |
| AGPL（BEN）与 LGPL（ForTMR2 lib） | 许可兼容性 | LGPL 可被 AGPL 工程链接，保留声明即可 |
| LSPosed 与 Vulkan 层共存 | 注入时序冲突 | 明确加载顺序：Xposed 入口 → 运行时 → 渲染层 |

**待业主确认**：

1. `ForTMR4A` 与现有 `ForTMR` / `EFML4A` 的包名与分发关系（是否并存、是否共用私有后端）。
2. `mod.toml` 是否接受 `[ext.ben.*]` 扩展段（`efml-core` 会忽略未知字段，但需上游确认语义）。
3. `web/` 战斗分析是迁出独立仓还是直接停用。
4. `[target] abi` 是否只保留 `arm64-v8a`（BEN 现在也只支持 arm64）。

## 11. 本次已执行

- Git 分支 `main` → `better-endfield-next-mod-ForTMR4A`，并打回滚 tag `pre-fortmr4a`。
- 工作目录 `Better-Endfield` → `Better-Endfield-Next-mod-ForTMR4A`。
- `README.md` / `README.en.md`：标题与显示名改为 ForTMR4A，新增平台范围声明、ForTMR2 关系、
  构建链改为 Linux 宿主 → Android 产物，仓库布局表中标注 Windows 资产已停用。
- 本方案文档：`docs/workspace/FORTMR4A.md`。
