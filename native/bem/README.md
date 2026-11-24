# native/bem —— BEM 读取与业务校验（CRX 叶子）

BEM 包进入设备的**第一条业务链**：容器解析 → manifest 结构校验 → payload 引用/长度自洽。
用 Rust 写（复杂独立解析器，符合 CRX “Rust 写叶子”），通过 C ABI 暴露给导入链。

## 覆盖范围（切片 1）

对应 `docs/custom_model/BEM_FORMAT_SPEC.md`：

- §2 容器：magic、版本 1.0–1.5、40 字节头、file/manifest 尺寸、32 字节 payload 目录、
  偏移连续（不允许间隙/重叠/尾随）、codec 0/1、长度上限；
- §3 Manifest：schema=1、`package_id` 稳定 ID、能力集合（必需三项、未知/重复拒绝）、
  数组上限、嵌套深度；
- §5–§7 基本引用：`streams[].payload`、网格级/`draws[]` 级 `indices`、`textures[].payload`，
  以及 `vertex_count × stride`、`index_count × index_size` 的长度自洽；
- 解压全部 payload（zstd 单帧）并核对 `decoded_size`。

## 写入 / repack（切片 2）

- `write_bem_raw(minor, manifest, payloads)`：以 raw（codec=0）重写容器，包含全套头部/目录/边界校验；
- `BemPackage::repack_raw()`：读→写；保留 manifest 字节，payload 解出来后按 raw 重新封装；
- C ABI：`be_bem_repack`（raw）、`be_bem_repack_ex`（`compress`/`level`）；
- CLI：`be-bem repack <in> <out> [--zstd] [--level N]`、
  `be-bem pack <manifest.json> <payload-dir> <out> [--minor N] [--zstd]`；
- zstd 按规范“压缩有收益才用”：压缩后不小于原文就写 raw；实测 26.9 MB 包 repack 后 14.3 MB。

实测：对 26 MB 的 EFF 转换产物 repack 后与输入**逐字节一致**，且通过 `validate`。
zstd 压缩（codec=1）留给后续切片（`ruzstd` 只解码）。

**未覆盖（后续切片）**：像素格式字节表、材质覆盖/驱动器语义、1.4 多资源目标细节、
ZIP 容器（§14）、JSON 重复键检测。这些当前由 Python 参考实现 `tools/CustomModel/bem_v*.py`
与 C++ `bem.cpp` 继续承担，直到对应切片迁移完成。

## 用法

```bash
cd native/bem
cargo test
cargo run --bin be-bem -- inspect ../../android/app/src/androidTest/assets/installer-fixture.bem
cargo run --bin be-bem -- validate ../../android/app/src/androidTest/assets/installer-fixture.bem
```

C ABI：`native/shared/include/BetterEndfieldNext/BemPackage.h`
（`be_bem_validate` / `be_bem_info_json` / `be_bem_abi_version`）。

## Android 接入（可选）

```bash
./gradlew :app:assembleRelease \
  -Pandroid.injected.cmake.arguments=-DBETTER_ENDFIELD_NEXT_BEM_RUST=ON
```

## 与 ForTMR2 的关系

BEM 是 BEN 的创作物格式；进入设备后要转成 ForTMR2 的 `mod.toml` + IR（`native/mod`）。
本 crate 只负责“包是否合法、内容能否按声明读出”，不决定渲染后端。
