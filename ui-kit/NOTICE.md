# Endfield UI X · 来源与边界（NOTICE）

## 上游

- 设计令牌与设计原则的**主要依据**：[icaruszezen/endfield-ui](https://github.com/icaruszezen/endfield-ui)
  （**MIT License**，作者 icaruszezen）。本仓的 `ui-kit/tokens/endfield-ui-x.tokens.json`
  与 `ui-kit/android/res/` 中的颜色、字阶、圆角、动效取值，是对其 `theme.css` / 设计文档的
  **适配与精简**（改为 Android `dp/sp` 与资源名），不是原样复制组件库代码。
- 同类项目仅作调研参考，未复制代码：
  - [triplepiers/ArkScope](https://github.com/triplepiers/ArkScope)（鹰角风 UI 复现）
  - `reend-components`（npm，React 组件）
  - `ark-ui-skill`（UI 规范类 skill）

## 本仓原创部分

- `ui-kit/` 下的资源命名、Android 主题/样式/尺寸组织、预览图（SVG/PNG）；
- `docs/ui/ENDFIELD-UI-X.md` 的 Android 适配小节（游戏内面板约束、触控尺寸、绘制性能）。

这些原创内容随本仓以 **AGPL-3.0-or-later** 发布；其中源自上游 MIT 的取值在 MIT 下同样可用。

## 不含官方素材

本目录**不包含**《明日方舟：终末地》的截图、立绘、Logo、图标、界面贴图或字体文件。
颜色与字号是**测量/归纳得到的数值**，不构成素材复制。

需要本机对照真实界面时，用 ForTMR 主仓（EFML4A）的纯 C VFS 工具抽取到本机临时目录
（`fortmr vfsfull` / `dds2png` / `imgtool`），**产物不得提交到本仓库或任何公开分发**。
见 `docs/ui/ENDFIELD-UI-X.md` 第 6 节。

## 商标

《明日方舟：终末地》及其素材版权归鹰角网络（Hypergryph）所有。本项目为独立非官方项目，
与其无关联，也不代表其认可。
