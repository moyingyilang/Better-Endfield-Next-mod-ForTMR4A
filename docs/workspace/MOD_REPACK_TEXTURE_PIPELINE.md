---
title: "Better Endfield Next mod ForTMR4A：mod 重打包与纹理全线路（基础版）"
subtitle: "从用户选中一个文件，到游戏里看到模型"
author: "ForTMR4A 分支文档"
date: "2026-11-27"
lang: zh-CN
toc-title: "目录"
revision: "v1.1（2026-11-27，按评审意见修订）"
---

# 一句话版本

一个 mod 包（BEM / ZIP / EFMI / EFF / RAR / 7z / tar）进来，先被**拆开**，能转的转成
**BEM**，再按当前设备能接受的**纹理格式**处理，最后写成一份游戏进程能读的**安装包**；
游戏启动时，注入模块把这份安装包读进内存，替换掉对应角色的模型与贴图。

下面把它拆成“最基础版本”，每一步只回答四个问题：**输入是什么、做什么、输出是什么、失败会怎样**。

---

# 第一部分：mod 重打包全线路

## 步骤 0：总览

```
用户选文件
   │
   ▼
[1] 拿到 URI / 文件
   │
   ▼
[2] 暂存：优先 fd 直读，否则复制一份
   │
   ▼
[3] 探测：BEM? ZIP? EFMI? EFF? RAR/7z/tar?
   │
   ▼
[4] 是压缩包就解包（libarchive，写进临时目录）
   │
   ▼
[5] 在解包结果里找“可导入的东西”：
      单独的 .bem 文件 / EFMI 目录 / EFF 目录
   │
   ▼
[6] EFMI / EFF → BEM（Converter4BENeXt，进程内 C++）
   │
   ▼
[7] BEM 结构解析与校验（40 字节头 + manifest + payload）
   │
   ▼
[8] 生成“安装计划”：外观、组件、骨架、材质、纹理
   │
   ▼
[9] 纹理预处理（见第二部分）
      · 支持 → 保留
      · 不支持 → ASTC 6x6/4x4
      · 无 ASTC → ETC2（待运行时接通）→ RGBA32 兜底
   │
   ▼
[10] 写安装区：generation 目录 + installed.bem + 索引
   │
   ▼
[11] 游戏进程加载：注入模块读 BEM，上传网格/材质/纹理
   │
   ▼
[12] 卸载 / 回退 / 更新：换 generation、删旧目录
```

## 步骤 1：拿到文件（URI）

- **输入**：用户在文件选择器里选的 `content://` URI。
- **处理**：Android 的存储访问框架只给 URI，不给路径；导入器必须通过
  `ContentResolver` 打开，或拿 `ParcelFileDescriptor`。
- **输出**：一个可读的流 / 文件描述符。
- **失败**：授权过期、文件被删、provider 不支持。表现为“无法打开包”，需要用户重新选择。
- **代码**：`BemSource.prepare`、`BemInstaller.startImport`。

## 步骤 2：暂存（能流式就不复制）

- **输入**：URI。
- **处理**：优先 `openFileDescriptor(uri, "r")` 把 fd 直接交给解包器；
  拿不到 fd 时才把内容复制到 `stage/source.bin`。
- **输出**：要么是“fd 直读通道”，要么是一个临时文件。
- **失败**：复制失败/空间不足。
- **代码**：`BemSource.prepare` 的 fast path 与 copy fallback；
  原生 `be_archive_probe_fd` / `be_archive_extract_fd`。

> 效率点：这一步是导入链路里最容易白花 IO 的地方。fd 直读省掉了“整包复制一次”。

## 步骤 3：探测类型

- **输入**：fd 或临时文件。
- **处理**：按内容判断，不信任扩展名：
  - BEM 头：`BEM\0PKG\0`；
  - ZIP / RAR / 7z / tar / gzip / bzip2 / xz / zstd：交给 libarchive 试读第一个条目；
  - EFMI：有 `mod.ini` + `Meshes/` 等目录结构；
  - EFF Format 3：`mod.ini` 里 `format=3` + `materials/*.effres`。
- **输出**：类型枚举。
- **失败**：都不匹配 → 回退到旧的“只认 BEM/ZIP”错误。
- **代码**：`BemSource.probe/probeArchive`、`be_source_probe`、`be_archive_probe`。

## 步骤 4：解包

- **输入**：压缩包（fd 或路径）。
- **处理**：libarchive 逐条目标解出；策略：
  - 拒绝绝对路径、`..`、符号链接/硬链接；
  - 拒绝加密条目；
  - 上限：10 万条目、解压后 8 GiB；
  - 缓冲 1 MiB。
- **输出**：临时目录 `stage/extracted/`（保留原目录层级）。
- **失败**：加密包、分卷 RAR、损坏包。
- **代码**：`native/convert/archive_extract.cpp`。

## 步骤 5：找到“可导入的东西”

- **输入**：解包目录（或直接给的 BEM 文件/EFMI 目录）。
- **处理**：深度优先（最多 3 层）寻找：
  1. EFMI/EFF 目录 → 交给步骤 6；
  2. `BEM\0PKG\0` 开头的文件 → 直接进入步骤 7。
- **输出**：一个 BEM 文件，或一个 EFMI/EFF 目录。
- **失败**：压缩包里没有模型 → “没有可导入的 BEM”。
- **代码**：`BemSource.findSource`。

## 步骤 6：EFMI / EFF → BEM

- **输入**：EFMI 目录（`mod.ini` + Meshes/Textures）或 EFF 目录（`mod.ini` + `.effres`）。
- **处理**：Converter4BENeXt 在**进程内**完成：
  - 读 `mod.ini` 找到替换目标（角色/部件/材质/贴图）；
  - 读 BEN catalog 把组件、骨骼映射到当前游戏；
  - EFMI：按 Component 合并骨架、顶点、索引；
  - EFF：解 `EFFMESH v6` 快照（位置/法线/切线/UV/权重/骨骼路径）；
  - 拼 BEM manifest：`meshes`、`textures`、`skeleton`、`appearances`；
  - 输出 BEM 二进制（或 ZIP 容器）。
- **输出**：`stage/converted.bem`。
- **失败**：catalog 不匹配角色、EFF 版本不支持、骨骼映射失败（某个部件会被跳过并打日志）。
- **代码**：`native/convert/source_convert.cpp` + `third_party/Converter4BENeXt/src/main.cpp`。

> 效率点：转换本身很快（宿主机实测 EFF 448 ms、EFMI 801 ms）；设备上的大头是解包与纹理重编码。

## 步骤 7：BEM 结构解析与校验

- **输入**：BEM 文件。
- **处理**：按 BEM 1.x 规范读：
  - 40 字节文件头（magic、版本、header 大小、总大小、manifest 偏移/长度、payload 数）；
  - JSON manifest（`meshes` / `textures` / `skeleton` / `appearances` / `target`）；
  - 32 字节 payload 目录项（偏移、大小、编码方式 raw/zstd）；
  - 校验长度自洽、流 stride、属性偏移、索引范围。
- **输出**：内存里的包模型（网格、材质、骨架、纹理描述）。
- **失败**：截断、版本不支持、payload 长度对不上、manifest 非法。
- **代码**：`native/modules/custom_model/bem.cpp`、`installer/install_jni.cpp`。

## 步骤 8：生成安装计划

- **输入**：解析后的 BEM。
- **处理**：把“包”翻译成“要安装什么”：
  - 这次要替换哪些角色/组件；
  - 选哪个外观（appearances，默认 default）；
  - 骨架如何映射到游戏骨骼；
  - 材质槽、贴图引用；
  - 是否保留本地副本、是否跳过校验等选项。
- **输出**：安装计划 + 校验报告（inspect 报告）。
- **失败**：目标角色不匹配、组件缺失、版本不兼容。
- **代码**：`BemInstaller.prepareImport/inspectNative`、manifest 校验逻辑。

## 步骤 9：纹理预处理

见第二部分。简单说：统计包里所有纹理格式 → 探测设备能采样的格式 → 不支持的转成
ASTC/RGBA32 → 写回 BEM 的 payload。

## 步骤 10：写安装区

- **输入**：处理后的 BEM。
- **处理**：
  - 新 generation 目录：`files/bem-installed/<generation>/installed.bem`；
  - 更新索引 `files/bem-index.json`；
  - 旧 generation 在确认新包可用后删除；
  - 可选：不保留本地副本时只留游戏进程用的私有副本。
- **输出**：游戏进程下次启动能读到的安装包。
- **失败**：空间不足、写入中断 → 保留旧 generation（回滚）。
- **代码**：`BemInstaller.installPrepared/commitIndex/discardGeneration`。

## 步骤 11：运行时加载

- **输入**：安装区里的 `installed.bem`。
- **处理**：LSPosed 注入到游戏进程后：
  - 模块启动时读 `bem-installed` 下的包；
  - BEM reader 解析网格/骨架/材质；
  - 替换游戏里对应角色的 Mesh、材质与贴图；
  - 支持热切换（重新加载 registry，不重启游戏）。
- **输出**：游戏里看到替换后的模型。
- **失败**：游戏版本不匹配、骨骼/材质映射错误 → 崩溃或原模型。
- **代码**：`native/modules/custom_model/module.cpp`、`native/modules/*`。

## 步骤 12：卸载 / 回退 / 更新

- 卸载 = 删 generation + 更新索引；
- 回退 = 指回旧 generation；
- 更新 = 新包安装成功后再删旧包，失败保留旧包。

---

# 第二部分：纹理全线路

## 步骤 T0：总览

```
源纹理（DDS: BC1/3/4/5/7，或 ASTC/RGBA）
   │
   ▼
[T1] 读进内存，记录 宽/高/mip/格式/sRGB
   │
   ▼
[T2] 写进 BEM：textures[] + payload（raw/zstd）
   │
   ▼
[T3] 导入时统计格式分布（format census）
   │
   ▼
[T4] 设备能力探测：ASTC? BC1/3/4/5/7? 最大尺寸?
   │
   ▼
[T5] 决策：全部支持 → 原样保留
             有任一不支持 → 整个包重编码
   │
   ▼
[T6] 解码到 RGBA8（BC/ASTC → 像素）
   │
   ▼
[T7] 需要时修复法线（BC5/xy-unorm → 重建 Z）
   │
   ▼
[T8] 重编码（ASTC 6x6/4x4 多线程；无 ASTC 用 RGBA32）
   │
   ▼
[T9] 写回 BEM payload / manifest / checksums
   │
   ▼
[T10] 游戏上传纹理，GPU 采样
```

## T1：源纹理是什么格式

| 常见格式 | 含义 | Android 常见支持情况 |
|---|---|---|
| BC1 | DXT1，含 1 位 alpha | 部分 Adreno 支持，Mali/PowerVR 多数不支持 |
| BC3 | DXT5，插值 alpha | 同上 |
| BC4 | 单通道 | 同上 |
| BC5 | 双通道（常用法线） | 同上 |
| BC7 | 高质量 RGBA | 更少见，通常只有桌面/部分 Adreno |
| ASTC | 4x4 / 5x5 / 6x6 | Android 主流（GLES 3.2 核心） |
| RGBA32 | 未压缩 | 所有设备 |

BEM 里的纹理描述（manifest 的 `textures[]`）：

```json
{ "width": 2048, "height": 2048, "mips": 12, "format": 25,
  "srgb": true, "original_name": "T_actor_body_d", "payload": 7 }
```

`format` 是 BEM 的枚举（RGBA32=4、BC1=10、BC3=12、BC7=25、BC4=26、BC5=27、
ASTC_4x4=48、ASTC_5x5=49、ASTC_6x6=50）。

## T2：纹理如何存进 BEM

- 纹理原始字节放进 payload 区；manifest 只存描述；
- payload 可选择 raw 或 zstd（导入时读取、安装时解压）；
- 一个纹理一个 payload，或按需共享。

## T2b：BEM format ↔ Unity TextureFormat ↔ 运行时支持

| BEM `format` | Unity `TextureFormat` | 含义 | 当前 BEM 运行时 |
|---|---|---|---|
| 4 | RGBA32 | 未压缩 | ✅ |
| 63 | R8 | 单通道 | ✅ |
| 10 | DXT1/BC1 | 1-bit alpha | ✅ |
| 12 | DXT5/BC3 | 插值 alpha | ✅ |
| 25 | BC7 | 高质量 RGBA | ✅（需设备支持） |
| 26 | BC4 | 单通道 | ✅ |
| 27 | BC5 | 双通道法线 | ✅ |
| 45 / 46 / 47 | ETC2_RGB / RGBA1 / RGBA8 | ETC2 家族 | ❌ 未接入（见状态表） |
| 48 / 49 / 50 | ASTC_4x4 / 5x5 / 6x6 | ASTC 家族 | ✅ |

> 这张表是**校验清单**：任何新转码目标都必须同时满足“BEM reader 认 + 运行时能上传 +
> Unity/游戏支持”三件事，缺一不可。ETC2 目前卡在“BEM reader 认不了”。

## T3：格式统计（census）

导入前先数一遍：`{BC7: 12, BC5: 3, RGBA32: 1}`。
这决定“要不要重编码”，而不是逐张纹理现场判断——避免包做到一半才发现格式不支持。

## T4：设备能力探测

在安装 worker 里用 EGL/GLES 探测（不碰游戏 GL 上下文）：

| 能力 | 探测方式 |
|---|---|
| BC1/BC3 | `GL_EXT_texture_compression_s3tc` |
| BC4/BC5 | `GL_EXT_texture_compression_rgtc` |
| BC7 | `GL_EXT_texture_compression_bptc` |
| ASTC | `GL_KHR_texture_compression_astc_ldr` / `GL_OES_texture_compression_astc`，或 GLES 3.2 |
| ETC2 | GLES 3.0 起步即核心；ES2 驱动看 `GL_OES_compressed_ETC2_pixel_formats` / `GL_EXT_texture_compression_etc2` |
| 最大尺寸 | `GL_MAX_TEXTURE_SIZE` |

代码：`TextureSupport.java`。

探测结果按 **设备指纹（`Build.FINGERPRINT` + SDK）** 缓存，后续导入不再创建 EGL 上下文；
驱动升级/换机时指纹变化会自动重新探测。

## T5：决策（关键，v1.1 修订）

决策分两层：**格式能力**决定“能否保留”，**策略**决定“逐张还是整包”。

### 5.1 格式能力判定

1. 统计包内纹理格式（census）：例如 `{25:12, 27:3, 4:1}`；
2. 逐格式查设备能力：
   - 全部支持 → **原样保留**（零重编码、零画质损失）；
   - 有格式不支持 → 进入 5.2 的转码决策。

### 5.2 转码目标（优先级）

```
有 ASTC（GLES 3.2 / KHR / OES）
  └─ 色彩/遮罩：ASTC 6x6
  └─ 法线：ASTC 4x4（或保留 XY 语义，见 T7）
无 ASTC 但有 ETC2（GLES 3.0 起）
  └─ ETC2：RGB=45 / RGBA1=46 / RGBA8=47   ← 编码器与运行时尚未接通（见第三部分状态表）
无 ASTC 无 ETC2
  └─ RGBA32 兜底（体积/带宽最大，仅小纹理或最后手段）
```

### 5.3 粒度策略（当前实现 vs 计划）

| 策略 | 说明 | 状态 |
|---|---|---|
| 整包一致重编码 | 任一张不支持就整个包重编码；逻辑简单、结果一致，但可能为一张 BC7 牺牲整包画质与耗时 | **当前实现** |
| 逐张/按材质组 | 支持的保留、不支持的才转码；法线/UI/遮罩分组 | 计划中（需要安装器按纹理粒度写回 + 一致性开关） |

> 评审建议的“按纹理/材质组逐张决策 + 整包一致性开关”是对的，但它依赖
> 安装器支持**部分纹理替换**（当前 `texture_install` 是整包级转换）。
> 这列为下一阶段首要工作，见文末状态表。

日志会打印：`texture decision: astc=1 etc2=1 bc1=0 ... formats=... rewrite=true`。

## T6：解码

把 BC/ASTC 压缩块解成 RGBA8。法线贴图额外保留“原始编码信息”，用于后续修复。

## T7：法线修复与语义（v1.1 明确）

**问题**：BC5 存 XY 两通道，Z 由 Shader 重建；重编码成 ASTC 后如果只存 XY、而游戏
Shader 按 RGB 采样，或反过来，就会语义不匹配。

**当前实现（已确定，不是猜测）**：

- 对检测为法线的纹理：解码 → 重建 Z → 归一化，存成 **RGB 法线**；
- manifest 写 `semantic = "normal"`、`normal_encoding = "xyz-unorm"`；
- 运行时 BEM reader 按 `xyz-unorm`（RGB 三通道法线）处理，和游戏 Android 原生
  ASTC 法线贴图的语义一致；
- 非法线纹理完全不动。

也就是说：**ASTC 里存的是重建后的 RGB 法线，不保留 BC5 的双通道语义**；
若将来要保留双通道（XY + Shader 重建），需要同时改 `normal_encoding` 和游戏 Shader 契约，
不能只改一边。

## T8：重编码（v1.1 补充 sRGB/mip/对齐）

- ASTC：`astcenc`，质量 `ASTCENC_PRE_FAST`；
- **sRGB**：色彩纹理用 `ASTCENC_PRF_LDR_SRGB`，线性数据用 `ASTCENC_PRF_LDR`；
  与 BEM manifest 的 `srgb` 字段一致；
- **块大小**：色彩/遮罩 6x6，法线 4x4；
- **mip**：逐级重编码，不跳过任何一级；
- **非块对齐尺寸**：ASTC 编码器负责边缘块填充；BC 源要求宽高 4 的倍数（BEM 校验会拦）；
- 多线程：`min(4, CPU)`；
- 限制：单纹理 ≤ 64 MiB、边长 ≤ 8192；超限自动降 mip/缩放。

## T9：写回与校验

- 替换 payload 字节；
- 更新 `format`、`mips`、`srgb`、`original_name`；
- 重新计算 checksums；
- 安装器再次校验“长度自洽”。

## T10：运行时采样

游戏把 BEM 里的纹理数据上传 GPU。Unity 侧会用
`SystemInfo.SupportsTextureFormat` 再检查一次；如果格式不支持，运行时会拒绝上传。

---

# 第三部分：为什么 Android 上 BC 常常不可用？怎样让“设备支持 BC”？

## 先给结论

**BC 是否可用由 GPU + 驱动决定，应用层无法真正“让设备支持 BC”。**
能改的只有：**导入时把 BC 转成设备支持的格式**，或者**从源头用 ASTC 出包**。

## 为什么不能强开

1. BC（S3TC/BPTC/RGTC）是桌面 GPU 的压缩格式，移动 GPU 只有部分型号带解码硬件；
2. Android 游戏用 `SystemInfo.SupportsTextureFormat` 判断，最终取决于 GL/Vulkan 驱动能力；
3. 即使 hook 这个函数让它返回 `true`，GPU 上传时不支持的格式仍会报错/黑图/崩溃；
4. 驱动更新通常由 OEM 提供，App 无法安装驱动。

## 各平台现实情况

| GPU | BC 支持 |
|---|---|
| Adreno（部分型号） | 暴露 `GL_EXT_texture_compression_s3tc`（BC1/3）、`rgtc`（BC4/5），高端可能 `bptc`（BC7） |
| Mali / PowerVR | 基本不支持 BC，主打 ASTC/ETC2 |
| 桌面 N 卡/A 卡 | 支持 |

## 让“我们的导入器”适配（已实现）

1. **探测**：安装前用 EGL 探测 BC1/3/4/5/7、ASTC、最大尺寸；
2. **保留**：设备支持 BC → 原样保留，画质最好、零重编码；
3. **重写**：设备不支持 → ASTC 6x6/4x4（无 ASTC 则 RGBA32）；
4. **法线**：BC5 法线在重编码前先修复；
5. **可观测**：日志打印设备能力与决策，方便排查。

### 实现状态表（v1.1）

| 项 | 状态 | 说明 |
|---|---|---|
| EGL 能力探测（BC/ASTC/最大尺寸） | ✅ 已实现 | `TextureSupport` |
| ETC2 **能力**探测 | ✅ 已实现（仅日志/规划） | `etc2=` 字段，支撑决策树 |
| ETC2 **编码 + 运行时** | ⏳ 计划 | 需 ETC2 编码器（如 etcpak）+ BEM reader 支持 45/46/47 + 运行时上传映射 |
| 设备能力缓存 | ✅ 已实现 | 按 `Build.FINGERPRINT + SDK` 缓存，避免每次导入建 EGL 上下文 |
| 整包重编码 | ✅ 当前策略 | 简单一致，代价见 T5 |
| 逐张/材质组重编码 + 一致性开关 | ⏳ 计划 | 安装器需支持部分纹理写回 |
| 法线重编码语义 | ✅ 已明确 | RGB + `normal_encoding=xyz-unorm` |
| 转码结果缓存（源 hash+目标格式+块大小） | ⏳ 计划 | 重装/切换外观时可复用 |
| Vulkan 格式特性探测 | ⏳ 计划 | 游戏若走 Vulkan，GLES 扩展不完全等价；记录后用于决策 |
| 压缩比/空间预检 | ✅ 已实现 | 解压比 > max(512MiB, 200×包体) 拒绝；解包前检查可用空间 |
| 索引原子写 | ✅ 已实现 | `AtomicFile` + generation 目录 |
| 并发导入互斥 | ✅ 已实现 | `BemInstaller` 单 worker + synchronized `busy` |
| 崩溃恢复扫描 | ⏳ 计划 | generation + 索引已有回退基础，可加启动扫描 |

## 其它可行路线

| 方案 | 说明 | 代价 |
|---|---|---|
| 作者直接出 ASTC 包 | Android 原生格式，导入零转码 | 需要重做资源 |
| 保留双包（PC=BC、Android=ASTC） | 分享时按平台给包 | 资源体积 ×2 |
| 只在支持 BC 的设备用 BC 包 | 由探测决定，不保证全设备 | 兼容性有限 |
| 模拟器/桌面 | 桌面 GPU 支持 BC | 不是手机场景 |

## 不建议

- Hook `SystemInfo.SupportsTextureFormat` 返回 `true`：只能骗过判断，骗不过 GPU；
- 依赖“工程模式开 BC”：多数驱动没有这个开关。

---

# 附：代码位置速查

| 环节 | 文件 |
|---|---|
| 导入调度 | `android/app/src/main/java/dev/betterendfield/next/BemInstaller.java` |
| 回退/解包/转换 | `android/app/src/main/java/dev/betterendfield/next/BemSource.java` |
| 归档解包 | `native/convert/archive_extract.cpp`、`native/shared/include/BetterEndfieldNext/SourceArchive.h` |
| EFMI/EFF→BEM | `native/convert/source_convert.cpp`、`third_party/Converter4BENeXt/src/main.cpp` |
| BEM 读写 | `native/modules/custom_model/bem.cpp` |
| 纹理能力 | `android/app/src/main/java/dev/betterendfield/next/TextureSupport.java` |
| 纹理重编码 | `android/app/src/main/cpp/installer/texture_install.cpp` |
| 安装写入 | `BemInstaller` + `files/bem-installed`、`bem-index.json` |
| 运行时注入 | `native/modules/custom_model/module.cpp` |

# 附：性能基线（宿主实测）

| 输入 | 输出 | 耗时 | 环境/参数 |
|---|---|---|---|
| EFF `lvtsfw`（54 MB 目录） | 26 MB BEM | 448 ms | Termux aarch64 宿主，转换不涉及 ASTC |
| EFMI `edmin_maid`（39 MB 目录） | 73 MB BEM | 801 ms | 同上 |

> 设备上还要加：解包写盘、ASTC 重编码（`min(4,CPU)` 线程、`ASTCENC_PRE_FAST`、含全部 mip）、
> 安装写盘。设备实测基线待补（骁龙/天玑各一台，记录包大小/纹理数/峰值内存）。

设备上看：`logcat -s BetterEndfieldNext.Import BetterEndfieldNext.Install`，
关注 `stage=extract-fd / copy / convert` 与纹理重编码耗时。
