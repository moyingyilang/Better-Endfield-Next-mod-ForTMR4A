# 性能目标与导入链路基准

> 原则：**业务层效率优先**。CRX 的意义在于 C/Rust 主链路比 Python/Java 快，
> 任何新功能都要能回答“这一步为什么不能更快”。

## 当前导入链路（Android）

```
选择文件 → BemInstaller worker
  └─ BEM 直接读取（原生 read/seek）
  └─ 失败 → BemSource 回退
        ├─ openFileDescriptor → libarchive 直接 fd 流式解包（无整包复制）
        ├─ 拷贝回退（provider 不给 fd 时）
        ├─ 找 BEM 文件 / EFMI/EFF 目录
        ├─ Converter4BENeXt 进程内转 BEM
        └─ 回到 BEM 安装链（校验 → 纹理策略 → 写入安装区）
```

## 本机基线（宿主转换，2026-11-26）

| 输入 | 输出 | 耗时 | 吞吐 |
|---|---|---|---|
| EFF `lvtsfw`（54 MB 目录） | 26 MB BEM | 448 ms | ~120 MB/s 输入 |
| EFMI `edmin_maid`（约 39 MB 目录） | 73 MB BEM | 801 ms | ~50 MB/s 输入 |

转换本身不是瓶颈；设备上真正贵的是**解包写盘**和**纹理格式转换**。

## 目标（2026-11-27 设定）

| 指标 | 目标 | 验收方式 |
|---|---|---|
| 参考包端到端导入耗时 | 相比 v1 基线下降 **≥30%** | 同一台设备、同一包、`stage=` 日志，两轮取均值 |
| 纹理重编码范围 | 只转设备**不支持**的纹理，支持的逐张保留 | 日志 `converted_textures/kept_textures`、`texture_mode=original/mixed/astc/rgba32` |
| 峰值内存 | 不高于现状（同包同设备） | dumpsys meminfo / 进程 RSS 采样 |
| 画质/兼容性 | 支持 BC 的设备保留原 BC；混合格式可加载 | 真机加载 + 截图对比 |

## 本轮优化（2026-11-27）

| 优化 | 说明 | 预期收益 |
|---|---|---|
| **逐张纹理保留** | 包级“需要重写”只作为触发条件；native 转换按设备能力 mask **逐张跳过**已支持的格式（RGBA32/R8 永远保留；BC1/3/4/5/7/ASTC 按 mask） | 只有真正不支持的纹理才转码；混合格式包不再整包重编码，画质与耗时可降一个量级 |
| **ASTC 上下文复用** | 按 `(profile, block)` 缓存 `astcenc_context`，不再每张纹理分配/释放 | 省掉每纹理 context 分配开销，纹理越多收益越大 |
| **混合 texture_mode** | 转换报告新增 `converted_textures/kept_textures`，安装记录写 `original/mixed/astc/rgba32` | 可观测真实转码范围，便于验收 |
| **保留纹理零解码** | `RewriteBemTextures` 增加 `keep` 预判：设备支持的纹理直接复制原 payload，**不解码、不做格式检测**（BC 解码是大头） | 大部分纹理保留的包，CPU 时间显著下降 |
| **转换不落临时副本** | 转换已安装包时直接从 `bem-installed/<gen>/installed.bem` 读取，不再先复制一份到 stage | 省掉 70 MB 级包的一次整包写+读 |

## 已完成优化（2026-11-26）

| 优化 | 效果 |
|---|---|
| **fd 流式解包** `be_archive_probe_fd / be_archive_extract_fd` | 去掉 RAR/7z/ZIP 导入前的一次整包复制（读 + 写），大包可省几十 MB IO |
| libarchive 读缓冲 64 KiB → **1 MiB** | 减少系统调用次数，解包大文件更快 |
| 解包输出缓冲 64 KiB → **1 MiB** | 同上 |
| ASTC 转换线程 1 → **min(4, CPU)** | 设备不支持 BC 纹理时的重编码大幅加速 |
| 阶段计时日志 `BetterEndfieldNext.Import` | 设备上可直接看到 `stage=extract-fd/copy/convert ms=… bytes=…` |

设备验证：

```bash
su -c 'logcat -d -s BetterEndfieldNext.Import BetterEndfieldNext.Install'
```

## 下一批候选（按收益排序）

1. **纹理批量并行**：当前按纹理串行 + 单纹内部多线程；可改成纹理间并行（内存允许时），
   对 15+ 纹理的包收益最大；
2. **BEM 写入去重**：转换产物写一次、安装校验后再写一次；评估把校验+安装合并为一次流式；
3. **BEM 读取 mmap**：`bem.cpp` 目前 `ifstream` + seek/read，大包可改只读 mmap 减少拷贝；
4. **zstd 多线程解压**：BEM zstd payload 可 `ZSTD_decompressDCtx` 多线程（流式格式需评估）；
5. **Java→native 调度**：把“探测→解包→转换→安装”整条调度移进 C 主链路，减少 JNI/Java 状态机开销；
6. **冷启动/预览**：模型预览仍走 Java/原生独立加载，统一到同一 reader。

## 测量纪律

- 设备上以 `stage=` 日志为准，A/B 两组各两轮，记录机型/温度/包大小；
- 宿主基准用 `date +%s%N` 前后差（Termux 无 `/usr/bin/time`）；
- 不为了设参数而设参数：改动必须有前后对比，否则回退。
