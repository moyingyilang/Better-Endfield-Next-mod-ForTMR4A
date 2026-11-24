# mods/ —— ForTMR4A 的 mod 包约定

本目录只放**格式约定与参考**，不携带任何游戏资源。mod 包格式由
[ForTMR2](https://github.com/moyingyilang/ForTMR2) 的 `efml-core` 定义：

```
my-mod/
├── mod.toml          # format_version / id / target / backends + IR 段落
├── checksums.txt     # 可选的 sha256 清单
├── mesh/*.bin        # [[mesh.replace]] 引用的数据
└── texture/*.bin     # [[texture.replace]] 引用的数据
```

参考包：`third_party/ForTMR2/mods/example-mod`（合成数据，覆盖 IR 全部段落）。

## 校验

```bash
# 宿主侧（Rust 叶子）
cd native/mod && cargo run --bin be-mod-validate -- ../../third_party/ForTMR2/mods/example-mod

# 或上游 CLI（若已构建）
third_party/ForTMR2/native/target/release/efml --json validate third_party/ForTMR2/mods/example-mod
```

## BEN 专属能力的落点

`efml-core` 明确**忽略未知字段**，所以 BEN 的相机 / MMD / 语音 / 动作等能力用
`[ext.ben.*]` 段与 ForTMR 的渲染替换共存，不破坏上游 loader 的前向兼容：

```toml
# 渲染替换：ForTMR2 IR（会被 vulkan-layer / il2cpp-hook 后端消费）
[[mesh.replace]]
vertex_count = 3
source = "mesh/body.vertex.bin"

[mesh.replace.identify]
kind = "pipeline-hash"
hash = "a1b2c3d4e5f60718"

# BEN 扩展：Android 运行时的相机/语音等，不参与 ForTMR2 的 IR 校验
[ext.ben]
schema = 1

[ext.ben.camera]
free_camera = true
fov = 60.0

[ext.ben.voice]
language = "ja"
```

## 边界

- 不含游戏原始资源；BEM 转换到该格式的转换器是独立仓（P3）。
- VFS 解密、`.blc/.chk`、密钥不在公开仓（见 ForTMR2 `docs/PUBLIC-BOUNDARY.md`）。
- 当前 BEM 1.0–1.5 仍由 `tools/CustomModel` 处理；迁移后这里只保留格式说明与样本。
