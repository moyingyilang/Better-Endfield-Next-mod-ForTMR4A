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
