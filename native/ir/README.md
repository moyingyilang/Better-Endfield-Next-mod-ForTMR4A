# native/ir —— BEM → ForTMR2 mod.toml/IR（CRX 叶子）

BEN 的创作物格式（BEM）与 ForTMR2 运行时格式（`mod.toml` + IR）之间的结构转换。
BEM 不淘汰：本 crate 只做“BEM → IR”的单向转换，BEM 读取/写入/校验仍在 `native/bem`。

## 映射

| BEM | IR |
|---|---|
| `meshes[].streams[3]`（顶点流） | 交错为单顶点缓冲 `mesh/N.vertex.bin` + `[[mesh.replace.vertex_layout]]` |
| `meshes[].attributes [sem,fmt,dim,stream,offset]` | 合并偏移后的 `attributes[]`（语义名 + 顶点格式名 + location） |
| `meshes[].indices` / `draws[].indices` | `[[mesh.replace.submeshes]]`（`index_format` = U16/U32） |
| `meshes[]` 的顶点签名 | `identify = { kind = "vertex-signature", stride, attributes }` |
| `textures[]`（payload + original_name + format + srgb） | `[[texture.replace]]`，`identify = { kind = "asset-path", path = original_name }` |
| `appearances` / `bones` / 能力 / BEM 版本 | `[ext.ben.*]`（`efml-core` 忽略未知字段） |

## catalog 补全（`--catalog`）

传入 BEN catalog 目录（`tools/CustomModel/catalog`，按 `character_id` 字段匹配文件）后：

| 产出 | 依据 | 说明 |
|---|---|---|
| `[[skeleton.map]]` | 部件 `bone_names` + 网格 `bones` | 按出现顺序去重合并骨骼槽；`identify = unity-path`（catalog 对象路径首段，如 `chr_0016_laevat_uimodel`） |
| 网格 `identify` | `evidence.observations.*.path` | 优先 Unity 对象路径；无 catalog 时退回 `vertex-signature` |
| `[[material.override]]` | `materials` + `material_texture_properties` | 每个材质槽的贴图 → IR `slot`（`_BaseMap` 等）+ `texture/N.bin`；无绑定的空覆盖跳过 |

**仍未映射**：骨骼 bindpose、shader 名与关键字覆盖、形态键、1.5 驱动器、1.4 多资源目标。
材质 `identify` 目前用 donor 材质名（`asset-path`），skeleton 根路径是 catalog 对象路径首段的启发式。

## BEM 原生 ZIP 容器

`convert_bem_zip_to_mod` / CLI `convert-zip` / C ABI `be_ir_convert_bem_zip`：
按 BEM 规范 §14 读取（`manifest.json` + `payloads/NNNN.bin`，STORED/DEFLATE，可剥单一顶层目录），
在内存中重封装成 BEM 然后复用同一条转换链。`format_version` 缺失时按字段推断 1.0–1.5。
普通 mod ZIP（EFMI/EFF 压缩包）需要先解包，不在这一层处理。

## 用法

```bash
cd native/ir
cargo test
cargo run --bin be-ir -- convert <in.bem> <out-dir> [--json]
```

C ABI：`native/shared/include/BetterEndfieldNext/IrConvert.h`（`be_ir_convert_bem`）。
Android 可选开关：`-DBETTER_ENDFIELD_NEXT_IR_RUST=ON`。

## 自校验

转换最后一步用 `efml_core::ModPackage::open + validate` 检查产物；失败直接返回错误，
不产出半成品。`efml validate <out-dir>` 可独立复验。
