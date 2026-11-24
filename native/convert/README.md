# native/convert —— 多格式源接入（BEM / EFMI / EFF3 / mod.toml）

ForTMR4A 的“**实时转换**”入口：导入时在进程内识别来源格式并转换，不再调用外部 CLI 或 Python。

```
BEM 文件 ────────────────┐
EFMI 目录（mod.ini） ──→ Converter4BENeXt（vendored，C++17）──→ BEM ──→ be-bem 校验
EFF Format 3（.effres）─┘
mod.toml（ForTMR2） ─────→ 交给 native/mod 的 efml-core 链（不转 BEM）
```

## 关键约束

- **BEM 不淘汰**：BEM 是原生格式；历史包、Converter4BENeXt 的输出、未来的转换结果都走同一条
  读取/校验链（`native/bem`）。
- **Converter4BENeXt 保留自己的 CLI 行为**：本仓只加了 C ABI 与 `lib` 构建目标，
  见 `third_party/Converter4BENeXt/VENDORED.md`。
- **进程内**：EFMI/EFF 转换不 spawn 子进程，Android 上直接链接静态库。

## C ABI

`SourceConvert.h`：

| 函数 | 说明 |
|---|---|
| `be_source_probe(path)` | 返回 `BE_SRC_BEM / MOD_TOML / EFMI / EFF3 / ZIP / UNKNOWN` |
| `be_source_convert_to_bem(src, catalog, out, err, cap)` | 转换到 BEM；BEM 源直接复制/校验，mod.toml 交 IR 链 |
| `be_source_abi_version()` | 适配层 ABI 版本 |

## 构建

Android（可选）：

```bash
./gradlew :app:assembleRelease \
  -Pandroid.injected.cmake.arguments=-DBETTER_ENDFIELD_NEXT_CONVERT=ON
```

宿主自检（Termux/clang 实测）：

```bash
C4=third_party/Converter4BENeXt
clang++ -O2 -std=c++17 -DC4BEN_NO_MAIN -I$C4/src -I$C4/src/third_party -c $C4/src/main.cpp -o /tmp/c4ben_lib.o
clang  -O2 -std=c11 -I$C4/src/third_party -c $C4/src/third_party/cJSON.c -o /tmp/cJSON.o
clang++ -O2 -std=c++17 -I native/convert -I native/shared/include -I $C4/src -c native/convert/source_convert.cpp -o /tmp/source_convert.o
clang  -O2 -I native/convert -c native/convert/tests/probe_harness.c -o /tmp/probe_harness.o
clang++ /tmp/*.o -o /tmp/be-convert-probe -lz
```

## 已验证（2026-11-24，Termux / arm64）

| 来源 | 探测 | 转换 | BEM 校验（`be-bem`） |
|---|---|---|---|
| BEM 夹具 `installer-fixture.bem` | `BEM` | 无需转换 | `v1.0` 通过 |
| EFMI `edmin_maid` | `EFMI` | 73 MB BEM，9 mesh / 18 texture / 54 payload | `c4ben.chr_0003_endminf`，76.4 MB decoded，通过 |
| EFF3 `lvtsfw/lvt` | `EFF3` | 26 MB BEM，4 mesh / 5 texture / 21 payload | `c4ben.chr_0016_laevat`，26.9 MB decoded，通过 |
| ForTMR2 `mods/example-mod` | `mod.toml` | 不转 BEM（IR 链） | — |

> 校验过程中发现并修复了 vendored 转换器的一个真实缺陷：EFF 路径 stream1 实际写 24 字节/顶点
> （UV 8B + color 16B），但 manifest 在无色时声明 stride=8，违反 BEM 规范的长度自洽；
> 已改为固定 24 并始终声明 color 属性，修复后 EFF 输出通过校验。
