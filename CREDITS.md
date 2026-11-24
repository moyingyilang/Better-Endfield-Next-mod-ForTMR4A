# 致谢 / Credits

> 这个仓库是一个 **fork**，并且特意保留 fork 关系：来源和致谢不该被“改名”抹掉。

## 最重要的：BEN 原项目

[**Better Endfield Next**](https://github.com/Dr-hydra/Better-Endfield)（作者 **Dr-hydra**，AGPL-3.0-only）
是整个 ForTMR4A 的基础，也是本项目**最需要感谢**的对象：

- 本仓库直接 fork 自它；原生模块、Host/模块 ABI、BEM 规范与工具链、Android JNI/LSPosed 壳、
  大部分文档与研究记录都源自 BEN 原项目；
- ForTMR4A 只做平台收敛（Linux 宿主 + Android 产物、移除 Windows）与 CRX 业务链改造，
  **没有 BEN 就没有这个分支**；
- BEN 原项目会持续作为上游保留：`https://github.com/Dr-hydra/Better-Endfield`
  （保留 fork 网络也正是为了持续显示这一来源）。

特别感谢 Dr-hydra 与 BEN 的所有贡献者。

## 同生态 / 上下游项目

| 项目 | 作者 / 来源 | 与本项目的关系 |
|---|---|---|
| [Better Endfield Next（BEN）](https://github.com/Dr-hydra/Better-Endfield) | Dr-hydra | **fork 上游**，AGPL-3.0-only |
| [E Mod Loader](https://github.com/Dr-hydra/Endfield-Mod-Loader) | Dr-hydra | 第三方模块/第一人称迁移去向 |
| [ForTMR](https://github.com/moyingyilang/ForTMR)（原 EFML4A） | moyingyilang | EFMI/EFF 逆向与 VFS 工具链、`LANGUAGE.md`（CRX）来源 |
| [ForTMR2](https://github.com/moyingyilang/ForTMR2) | moyingyilang | `efml-core` / `efml-cli` / `lib` / Vulkan 层（submodule） |
| [Converter4BENeXt](https://github.com/moyingyilang/Converter4BENeXt) | moyingyilang | EFMI/EFF → BEM 转换器（submodule，含本仓回推的 C ABI 与 EFF 修复） |
| [endfield-ui](https://github.com/icaruszezen/endfield-ui) | icaruszezen（MIT） | Endfield UI X 令牌与设计原则的主要依据 |
| [JMNeXt](https://github.com/moyingyilang/JMNeXt) / android-termux-builders4JMNeXt | moyingyilang（MPL-2.0） | `termux/` 构建链的改编来源 |
| [EIEM](https://github.com/Sasye/EIEM) | Sasye（AGPL-3.0） | VMD 解析/相机采样参考与源码快照（见 `THIRD_PARTY_NOTICES.md`） |
| [ArkScope](https://github.com/triplepiers/ArkScope)、`reend-components` | 社区 | 鹰角/终末地风格 UI 调研参考 |

## 第三方库

MinHook、Dobby、zstd、astcenc、miniz、nlohmann/json、bcdec、cJSON、7-Zip、Inno Setup 等，
各自保留原有许可证与来源说明；详见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) 与各目录内 LICENSE/NOTICE。

## 游戏与社区

《明日方舟：终末地》及全部官方素材版权归 **鹰角网络（Hypergryph）** 所有。
本项目是非官方、非营利的社区项目，与鹰角网络无关联，也不代表其认可。
感谢所有 BEM/MMD 创作者与测试者的反馈——没有你们的包和实测，导入链的校验规则不会这么具体。
