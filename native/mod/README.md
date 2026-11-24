# native/mod —— ForTMR2 mod 格式桥（Rust 叶子）

`be-mod-ffi` 把 ForTMR2 的 [`efml-core`](../../third_party/ForTMR2/native/crates/efml-core)
编成 C ABI，供 Android 侧的导入/校验流程使用。按 CRX 与 ForTMR2 `LANGUAGE.md`：

- **Rust 只做叶子**：解析/校验 `mod.toml` 与 IR，不接管 Android 主链路；
- **C ABI 是唯一界面**：见 [`ModPackage.h`](../shared/include/BetterEndfieldNext/ModPackage.h)；
- **`panic = "abort"`**：不允许 unwinding 穿过 FFI；入口先校验空指针/UTF-8；
- **上游 `efml-core` 保持 `forbid(unsafe_code)`**：`unsafe` 只在 `src/lib.rs` 的薄 shim 里。

## 宿主侧自检（不需要 NDK）

```bash
cd native/mod
cargo run --bin be-mod-validate -- ../../third_party/ForTMR2/mods/example-mod
```

## Android 侧（可选链接）

需要 Rust + `aarch64-linux-android` target，并在 Gradle 里传入：

```bash
./gradlew :app:assembleRelease -Pandroid.injected.cmake.arguments=-DBETTER_ENDFIELD_NEXT_MOD_RUST=ON
```

`native/mod/CMakeLists.txt` 会用 NDK 的 clang 作为 cargo linker 产出
`libbe_mod.a`，并以 `be_mod` 静态库暴露给 `betterendfieldnext_android`。

## 边界

- 公开仓只做「拿到 mod 包之后怎么校验/读取」；VFS 解密、`.blc/.chk`、密钥留在私有仓。
- BEN 特有的相机/语音/动作等能力不是 IR 的一部分，写在 `mod.toml` 的 `[ext.ben.*]`
  段（`efml-core` 明确忽略未知字段），见 [`mods/README.md`](../../mods/README.md)。
