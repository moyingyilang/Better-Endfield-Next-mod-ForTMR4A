# 业务链 CRX 重写台账

> 目标：把 ForTMR4A 的**业务链实现**迁到 CRX（C 主链路 / Rust 叶子 / C++ 宿主），
> 逐步清零 Python / JavaScript 实现；C# 只读参考、不新增实现。
> 规则来源：[ForTMR2 `LANGUAGE.md`](https://github.com/moyingyilang/ForTMR2/blob/main/LANGUAGE.md)。
> 状态：2026-11-24 起分批执行，**按“段”迁移、每段能用测试证明等价**，不做一次性大爆炸。

## 0. 硬约束：BEM 不淘汰

BEM 是 ForTMR4A 的原生创作物格式，**不因为接入 ForTMR2 / EFF 而退役**：
BEM 文件、Converter4BENeXt 的输出、以及未来所有来源的转换结果，统一进入同一条
BEM 读取/校验链（`native/bem`），再进入 IR/运行时。多格式只增加“入口”，不减“主线”。

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
| 1 | **BEM 包读取/校验**（导入第一关） | Python `tools/CustomModel/bem_v*.py`（写入/校验参考） + C++ `native/modules/custom_model/bem.cpp` | **Rust 叶子** `native/bem` + C ABI `BemPackage.h` | ✅ 切片 1：容器/结构/payload 引用/长度自洽 + zstd 解压校验 |
| 1b | **多格式源接入 + 实时转换**（BEM / EFMI / EFF3 / mod.toml） | Converter4BENeXt（C++17，submodule，pin `6fcd24f`）+ Python 工具链 | **C++ 宿主** `native/convert`（vendored converter，C ABI）+ C 探测/分发 | ✅ 切片 2：进程内转换，BEM 主线不变 |
| 1c | **统一导入入口（编排）** | 无（此前要手动串转换器 + 工具） | **C 主链路** `native/pipeline`（`be_source_to_mod`） | ✅ 切片 3.5：源探测→转换→BEM→IR；含 BEM ZIP；JNI `BemConverter`（未设备验证） |
| 2 | BEM 写入/合成（组合、材质、驱动器） | Python `bem_v1.py`、`bem_compose*.cpp` | Rust 叶子（与 1 同 crate）+ C 编排 | 🟡 切片 2 进行中：写入器/repack 完成（raw，读→写逐字节一致）；合成/材质/驱动器待做 |
| 3 | BEM → `mod.toml`/IR 转换 | 尚无（P3） | Rust 叶子（依赖 `efml-core`）+ C 编排 | ✅ 切片 3（本次）：`native/ir`，生成后用 efml-core 自校验 |
| 4 | 资源编解码（zstd / miniz / ASTC / BCn） | C（vendored）+ C++ 封装 | C 主链路 + Rust 叶子（新代码） | ⏳ 切片 4 |
| 5 | 安装/更新/回退状态机 | Java `BemInstaller` / `MmdInstaller` + Python 生成脚本 | **C 主链路** + Java 只做 UI/框架调用 | ⏳ 切片 5 |
| 6 | 模块配置分发与解析 | Java `ModuleConfigurations` → C++ 模块文本解析 | C 主链路（统一配置格式）+ C++ 宿主 | ⏳ 切片 6 |
| 7 | 运行时功能（相机/语音/动作/UI/媒体屏） | C++ 模块（多平台共用 TU） | 保留 **C++ 宿主**；可移植算法抽 C/Rust 叶子 | ⏳ 切片 7（以抽离为主，不重写 UI 逻辑） |
| 8 | 战斗/寻访数据管线 | Windows 模块已归档；`web/` 待迁出 | 不在本分支范围 | ⏸ 随 P4 |
| 9 | 构建/资源生成 | Python `scripts/*.py` + PowerShell（已归档） | shell + Rust CLI | ⏳ 切片 8 |
| 10 | 设置界面 | Java（程序化 UI） | Java 平台壳（不在重写范围）；视觉走 Endfield UI X | — |

## 2b. 切片 2 交付物：多格式源接入（本次）

- `third_party/Converter4BENeXt/`：上游 `24c3a8a` vendored；抽出 `convertEfmi()`、
  新增 C ABI（`c4ben_probe/convert/convert_profile`）、`-DC4BEN_NO_MAIN` 库模式与 `lib` 目标；
- `native/convert/`：`be_source_probe` / `be_source_convert_to_bem` C ABI，
  识别 BEM / mod.toml / EFMI / EFF3 / ZIP，EFMI/EFF 进程内转 BEM；
- Android CMake 可选开关 `BETTER_ENDFIELD_NEXT_CONVERT`（默认 OFF）；
- 实测：EFMI `edmin_maid` → 73 MB BEM（9 mesh/18 texture/54 payload）→ `be-bem` 通过；
  EFF3 `lvtsfw/lvt` → 26 MB BEM（4 mesh/5 texture/21 payload）→ `be-bem` 通过；
- 发现并修复转换器 EFF 路径 stream1 stride/属性不一致缺陷（应回推上游）。

## 2c. 切片 2 进展：BEM 写入 / repack

- `native/bem` 新增 `write_bem_raw` / `BemPackage::repack_raw` / C ABI `be_bem_repack` /
  CLI `repack`、`pack`；覆盖头部、目录连续性、边界与 manifest JSON 合法性；
- 8 项 `cargo test` 通过（新增 3 项：写入后重开、夹具 repack 往返、非法 manifest 拒绝）；
- 实测 26 MB EFF 转换产物 repack 后与输入**逐字节一致**，`validate` 通过；
  这给“读→改→写”与后续 BEM→IR 提供统一出口。

## 2d. 切片 3 交付物：BEM → mod.toml/IR（本次）

- `native/ir`（`be-ir`，Rust 叶子，依赖 `efml-core` + `be-bem`）：
  - 3 条 BEM 顶点流（position/normal/tangent、uv/color、skin）交错为 IR 顶点缓冲；
  - BEM 网格级/按 draw 的索引 → IR 子网格；
  - 贴图 payload 落盘 + Unity TextureFormat → IR 格式提示；
  - 识别策略：网格用 `vertex-signature`（stride + 语义组合），贴图用 `asset-path`（`original_name`）；
  - BEM 特有信息写 `[ext.ben.*]`（外观数、骨骼数、源包 id、BEM 版本），`efml-core` 忽略；
  - 生成后调用 `efml-core` 的 `ModPackage::open + validate` 自校验，不合法不产出；
  - C ABI `be_ir_convert_bem`；CLI `be-ir convert`。
- 未映射（留后续）：骨骼 bindpose、材质槽名/关键字覆盖、形态键、驱动器、1.4 多资源目标。

## 2e. 切片 3.5 交付物：统一导入入口（本次）

- `native/pipeline`（C 主链路）：`be_source_to_mod(src, catalog, out_mod_dir, err, cap)`；
  - BEM → 直接 `be_ir_convert_bem`；
  - EFMI/EFF → 进程内转 BEM（临时文件在输出目录内，完成后删除）→ 同一条 IR 链；
  - `mod.toml` 来源返回 UNSUPPORTED（已是原生格式）；ZIP 暂不接入，先解包；
- C ABI `SourcePipeline.h`；宿主 CLI `be-source`；Android 可选开关 `BETTER_ENDFIELD_NEXT_PIPELINE`
  （自动拉入 convert + ir）；
- 实测：fixture / EFF3 `lvtsfw` / EFMI `edmin_maid` 三种来源各一次调用产出 mod 目录，
  全部通过 `efml validate`。

## 2f. 追加：BEM 原生 ZIP 容器（规范 §14）

- `be-ir` 新增 `convert_bem_zip_to_mod` / C ABI `be_ir_convert_bem_zip` / CLI `convert-zip`：
  读 `manifest.json` + `payloads/NNNN.bin`（STORED/DEFLATE、可剥单一顶层目录），
  缺失 `format_version` 时按字段推断 1.0–1.5，内存重封装后复用 IR 链；
- 统一入口 `be_source_to_mod` 对 `PK` 魔数分派到这条路径；普通 mod ZIP 仍要求先解包；
- 实测：Converter4BENeXt 直接输出的 26 MB `lvtsfw.bem.zip` → mod 目录 → `efml validate` 通过。

## 3. 切片 1 交付物

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
| 已迁到 CRX 的业务段 | 4.5 / 10（BEM 读/写/校验、多格式源接入/实时转换、BEM→IR、统一入口、BEM ZIP） |
| 新增 Rust 叶子 | `native/mod`（mod.toml/IR 校验）、`native/bem`（BEM 读取/校验） |
| 新增 C 主链路 | `efml4a_source`（资源源抽象，消费 ForTMR2 `lib/`） |
| 新增 C++ 宿主适配 | `native/convert`（源探测/分发，C ABI 出边界） |

## 6. 非目标

- 不重写 Android 平台壳（Xposed/Activity/Java UI）——这些是框架约束，不是“业务链”；
- 不为 Windows 恢复任何实现（已归档）；
- 不把 Endfield UI X 的组件实现塞进业务链（UI 归 `ui-kit/`）。
