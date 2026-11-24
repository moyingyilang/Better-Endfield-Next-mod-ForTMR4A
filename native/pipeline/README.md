# native/pipeline —— 统一导入入口（CRX 的 C 主链路）

一条 C ABI 完成 ENTIRE 导入链：

```
BEM      ───────────────────────────────────────┐
BEM ZIP  ──（原生容器解包）──────────────────────┤
EFMI     ─┐                                     ├─→ be-ir ─→ mod 目录（mod.toml + 数据 + checksums）
EFF3     ─┴─ Converter4BENeXt（进程内）→ BEM ────┘
mod.toml（已是原生）→ 直接返回 UNSUPPORTED
```

## 职责划分（CRX）

| 层 | 文件 | 职责 |
|---|---|---|
| C 主链路 | `source_pipeline.c` | 源探测、临时文件生命周期、错误传播、输出目录 |
| C++ 宿主 | `native/convert` | EFMI/EFF → BEM（vendored 转换器，C ABI） |
| Rust 叶子 | `be-bem` / `be-ir` | BEM 校验、BEM→mod.toml/IR、checksums |

## 接口

`native/shared/include/BetterEndfieldNext/SourcePipeline.h`：

```c
int be_source_to_mod(const char *src, const char *catalog_dir,
                     const char *out_mod_dir, char *err, unsigned int err_cap);
```

返回：`0` 成功 / `1` 失败 / `2` 参数错误 / `3` 不支持（例如来源已是 `mod.toml`）。

- BEM 原生 ZIP 容器（`manifest.json` + `payloads/NNNN.bin`，可被单一顶层目录包裹）由 `be-ir`
  内部解包，再走同一条 IR 链；
- 普通 mod ZIP（EFMI/EFF 压缩包）暂不支持：先解包成目录再传。

## 宿主 CLI

```bash
cd native/pipeline
# 需要 clang/clang++、zlib；并已构建 native/ir 的 libbe_ir.a 与 native/convert 的对象文件
# 构建与测试命令见 docs/workspace/CRX-REWRITE.md 的“统一入口”小节
./be-source <src> <catalog|-> <out-mod-dir>
```

## Android

`-DBETTER_ENDFIELD_NEXT_PIPELINE=ON` 会自动拉入 `native/convert` 与 `native/ir`
（若已单独开启对应开关则复用），并把 `be_pipeline` 链进 `betterendfieldnext_android`。
