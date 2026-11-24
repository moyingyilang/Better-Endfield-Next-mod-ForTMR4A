# Converter4BENeXt（vendored）

本目录是 [Converter4BENeXt](https://github.com/moyingyilang/Converter4BENeXt) 的**进程内副本**，
用于 ForTMR4A 的“多格式源实时转换”：

- 上游 commit：`24c3a8ad9528042a9a21eefe0e7daeb4cd8df9b6`
- 许可：AGPL-3.0-or-later（见 `LICENSE`、`NOTICE`）
- 相对于上游的改动（保持 CLI 行为不变）：
  1. 从 `main()` 中抽出 `convertEfmi()`，让 EFMI 主流程可复用；
  2. 新增 `src/c4ben.h` C ABI（`c4ben_probe` / `c4ben_convert` / `c4ben_convert_profile` / `c4ben_version`）；
  3. `main()` 用 `#ifndef C4BEN_NO_MAIN` 包住，库模式用 `-DC4BEN_NO_MAIN` 编译；
  4. `Makefile` 增加 `lib` 目标（`build/libc4ben.a`），`all` 同时产出 CLI 与静态库；
  5. **修复 EFF 路径 stream1 的 stride/属性不一致**：s1 实际是 UV(8B)+color(16B)=24B/顶点，
     原实现在无顶点色时声明 stride=8、且不声明 color 属性，违反 BEM 规范的长度自洽；
     现固定声明 24 与 color 属性（该修复应回推上游）。

## 上游状态

C ABI 与 EFF stride 修复已经推送到上游分支：

- 分支：`agent/crx-lib-api`
- commit：`6fcd24f39897a3916f13a7d6a2f17513c5c79999`
- PR：https://github.com/moyingyilang/Converter4BENeXt/pull/new/agent/crx-lib-api

在该分支合入 `main` 之前，本仓保留 vendored 副本以保证可独立构建；
合入后改为 submodule 并删除本目录（见 `docs/workspace/CRX-REWRITE.md`）。

## 同步

```bash
# 重新拉取上游并重放 4 处改动（见上），或等上游合入库接口后用 submodule 替换
git -C ../../../.. submodule add https://github.com/moyingyilang/Converter4BENeXt.git third_party/Converter4BENeXt
```

## 构建

```bash
make            # build/c4ben + build/libc4ben.a
make lib        # 只出静态库
```
