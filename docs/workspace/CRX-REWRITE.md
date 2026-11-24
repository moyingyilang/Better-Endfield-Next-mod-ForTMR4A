# 业务链 CRX 重写台账

> 目标：把 ForTMR4A 的**业务链实现**迁到 CRX（C 主链路 / Rust 叶子 / C++ 宿主），
> 逐步清零 Python / JavaScript 实现；C# 只读参考、不新增实现。
> 规则来源：[ForTMR2 `LANGUAGE.md`](https://github.com/moyingyilang/ForTMR2/blob/main/LANGUAGE.md)。
> 状态：2026-11-24 起分批执行，**按“段”迁移、每段能用测试证明等价**，不做一次性大爆炸。

## 1. CRX 在业务链上的落点

| 层 | 语言 | 业务链职责 | 禁止 |
|---|---|---|---|
| 主链路 | **C11/C17** | 导入编排、资源生命周期、CLI dispatch、包状态机、安装/回退 | 运行时/GC/异常；隐藏分配 |
| 叶子 | **Rust** | 格式解析与校验、压缩/哈希、复杂但独立的解析器（BEM / mod.toml / IR） | 接管主循环、跨 FFI panic、热路径分配 |
| 宿主 | **C++17/20** | JNI、Hook broker、平台模块、复杂对象管理 | 跨 ABI 的 STL 类型/异常 |
| 平台壳 | Java / shell | Xposed 入口、Activity、构建脚本 | 业务实现 |

## 2. 业务链分段与状态

| # | 业务段 | 现实现 | CRX 目标 | 状态 |
|---|---|---|---|---|
| 1 | **BEM 包读取/校验**（导入第一关） | Python `tools/CustomModel/bem_v*.py`（写入/校验参考） + C++ `native/modules/custom_model/bem.cpp` | **Rust 叶子** `native/bem` + C ABI `BemPackage.h` | ✅ **切片 1（本次）**：容器/结构/payload 引用/长度自洽 + zstd 解压校验 |
| 2 | BEM 写入/合成（组合、材质、驱动器） | Python `bem_v1.py`、`bem_compose*.cpp` | Rust 叶子（与 1 同 crate）+ C 编排 | ⏳ 切片 2 |
| 3 | BEM → `mod.toml`/IR 转换 | 尚无（P3） | Rust 叶子（依赖 `efml-core`）+ C 编排 | ⏳ 切片 3 |
| 4 | 资源编解码（zstd / miniz / ASTC / BCn） | C（vendored）+ C++ 封装 | C 主链路 + Rust 叶子（新代码） | ⏳ 切片 4 |
| 5 | 安装/更新/回退状态机 | Java `BemInstaller` / `MmdInstaller` + Python 生成脚本 | **C 主链路** + Java 只做 UI/框架调用 | ⏳ 切片 5 |
| 6 | 模块配置分发与解析 | Java `ModuleConfigurations` → C++ 模块文本解析 | C 主链路（统一配置格式）+ C++ 宿主 | ⏳ 切片 6 |
| 7 | 运行时功能（相机/语音/动作/UI/媒体屏） | C++ 模块（多平台共用 TU） | 保留 **C++ 宿主**；可移植算法抽 C/Rust 叶子 | ⏳ 切片 7（以抽离为主，不重写 UI 逻辑） |
| 8 | 战斗/寻访数据管线 | Windows 模块已归档；`web/` 待迁出 | 不在本分支范围 | ⏸ 随 P4 |
| 9 | 构建/资源生成 | Python `scripts/*.py` + PowerShell（已归档） | shell + Rust CLI | ⏳ 切片 8 |
| 10 | 设置界面 | Java（程序化 UI） | Java 平台壳（不在重写范围）；视觉走 Endfield UI X | — |

## 3. 切片 1 交付物（本次）

- `native/bem/`（`be-bem`，Rust 叶子）：
  - 容器：magic / 版本 1.0–1.5 / 40B 头 / 尺寸 / 32B 目录 / 连续性与边界 / codec；
  - Manifest：schema、`package_id`、能力集合、数组上限、嵌套深度；
  - 引用：`streams[].payload`、网格级与 `draws[]` 级 `indices`、`textures[].payload`；
  - 长度自洽：`vertex_count × stride`、`index_count × index_size`；
  - 全 payload zstd 解压并核对 `decoded_size`；
  - C ABI：`be_bem_validate` / `be_bem_info_json` / `be_bem_abi_version`；
  - CLI：`be-bem inspect|validate`（JSON 可机读）；
  - 测试：合成包（最小合法/坏 magic/尺寸不符/悬空引用）+ 仓库夹具 `installer-fixture.bem`。
- Android CMake 可选开关 `BETTER_ENDFIELD_NEXT_BEM_RUST`（默认 OFF），产物 `libbe_bem.a`。

**未覆盖（显式留在切片 2+）**：像素格式字节表、材质/驱动器语义、1.4 多资源目标的字段互斥、
ZIP 容器（§14）、JSON 重复键检测、流式读取（当前整文件读入，>100 MB 包需要改 mmap/seek）。

## 4. 迁移方式（strangler，可回滚）

1. 新 CRX 实现与旧实现**并存**，通过构建开关/运行时 flag 选择；
2. 每段用现有测试数据对拍：Python 参考实现输出 vs 新实现输出（字段级比较）；
3. 对拍通过且设备回归通过后，才删除旧实现；
4. 旧实现删除前保留在 `legacy/`（不参与构建）。

## 5. 度量

| 指标 | 当前 |
|---|---|
| 业务链中的 Python 实现 | `tools/CustomModel` 89 文件 / 约 2.4 万行（BEM 相关），`scripts/` 生成器 |
| 已迁到 CRX 的业务段 | 1 / 10（BEM 读取/校验） |
| 新增 Rust 叶子 | `native/mod`（mod.toml/IR 校验）、`native/bem`（BEM 读取/校验） |
| 新增 C 主链路 | `efml4a_source`（资源源抽象，消费 ForTMR2 `lib/`） |

## 6. 非目标

- 不重写 Android 平台壳（Xposed/Activity/Java UI）——这些是框架约束，不是“业务链”；
- 不为 Windows 恢复任何实现（已归档）；
- 不把 Endfield UI X 的组件实现塞进业务链（UI 归 `ui-kit/`）。
