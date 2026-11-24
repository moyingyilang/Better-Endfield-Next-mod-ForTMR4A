# Endfield UI X · v0.1

> 面向 **ForTMR4A（Android）** 的 Endfield 风格 UI 规范与令牌。
> 非官方项目，与鹰角网络（Hypergryph）无关。**本仓库不包含任何官方截图、立绘、Logo、图标或字体文件。**
> 依据等级沿用社区惯例：**实测**（官网/公开页面测量）、**社区**（开源项目归纳）、**推断**（本仓适配建议）。

## 0. 名称与来源

| 项 | 内容 |
|---|---|
| 名称 | **Endfield UI X**（EUX），版本 v0.1 |
| 主要外部依据 | [icaruszezen/endfield-ui](https://github.com/icaruszezen/endfield-ui)（MIT）：设计文档 + Tailwind 令牌 + React 组件库 |
| 同类参考 | [triplepiers/ArkScope](https://github.com/triplepiers/ArkScope)（鹰角风组件复现）、`reend-components`（npm）、`ark-ui-skill` |
| 本文档用途 | 把这套视觉语言落成 **Android 可用**的令牌、组件规格与实现提示，服务 `android/` 的设置页/游戏内面板 |
| 本仓落地位置 | `ui-kit/tokens/`、`ui-kit/android/res/`、`ui-kit/NOTICE.md` |

这套风格可以概括为**工业编辑风**：纸白或烟黑的底、墨黑的字、一处高饱和的**信号黄**，再叠一层克制的印刷与测绘痕迹。
它不是“黑底黄字加切角”，而是有约束的排版系统。

## 1. 六条原则（按 Android 场景改写）

1. **纸与墨是主体，颜色是信号。** 默认亮色；暗色不是纯黑堆叠，而是 `#141414 → #1F1F1F → #282828` 的明度阶梯。信号黄只用于行动与强调，一屏不超过一处主行动。
2. **主角偏心，信息靠边。** 主标题/主数值不在正中；型号、页脚、状态、页码贴边或靠角，用等宽小字。
3. **形状有含义。** 直角是默认；切角表示“可操作/可展开”，圆角只给实机风格控件；圆 = 仪表/旋钮；不要用圆角卡片堆“通用感”。
4. **中文负责阅读，英文负责结构。** 中文用无衬线正文；英文标签、单位、编号用等宽或窄体大写字距 `0.05em`。
5. **纹理有预算。** 斜纹、底纹、角括号、测绘叠层是“一层”装饰，不叠加多层；每屏最多两种母题。
6. **动得短，动得硬。** 200–300ms，`ease-out`，无回弹；入场只做位移 4–8px 的淡入，不做滑屏。

## 2. 设计令牌

机器可读版本：[`ui-kit/tokens/endfield-ui-x.tokens.json`](../ui-kit/tokens/endfield-ui-x.tokens.json)；
Android 资源：[`ui-kit/android/res/`](../ui-kit/android/res/)。

### 2.1 语义颜色（亮 / 暗）

| 令牌 | 亮色 | 暗色 | 用途 |
|---|---|---|---|
| `surface` | `#FFFFFF` | `#141414` | 页面底 |
| `surfaceRaised` | `#FAFAFA` | `#1F1F1F` | 卡片/面板 |
| `surfaceSunken` | `#F2F2F2` | `#000000` | 凹陷区、输入槽 |
| `surfaceMuted` | `#E6E6E6` | `#282828` | 次级块、禁用底 |
| `surfaceInverse` | `#191919` | `#FAFAFA` | 反转带（标题带） |
| `ink` | `#191919` | `#FAFAFA` | 正文 |
| `inkSecondary` | `#626262` | `#B3B3B3` | 次要文字 |
| `inkTertiary` | `#8A8A8A` | `#8A8A8A` | 单位/注释 |
| `inkDisabled` | `#B3B3B3` | `#626262` | 禁用 |
| `line` | `#D9D9D9` | `#383838` | 1px 分隔线 |
| `lineStrong` | `#B3B3B3` | `#626262` | 选中/强调线 |
| `action` | `#FFFA00` | `#FFFA00` | 信号黄：主行动、选中 |
| `actionPressed` | `#EEEA00` | `#EEEA00` | 按下 |
| `onAction` | `#191919` | `#191919` | 信号黄上的字 |
| `accentInk` | `#6B5D00` | `#FFFA00` | 黄底以外的强调文字 |
| `control` | `#383838` | `#383838` | 中性控件 |
| `focus` | `#191919` | `#FFFA00` | 焦点环/焦点线 |
| `scrim` | `#80000000` | `#B3000000` | 遮罩 |

状态色（文字级对比 ≥ 4.5:1）：`danger #DE0909 / #F74040`、`success #007F43 / #00D872`、`warning #B74E00 / #FC6C00`、`info #25759F / #2D8FC2`。

角色/数据色（作填充，不作正文）：`notice #FC6C00`、`gain #BCE41D`、`data #2A84B4`、`region #06A2A2`、`alert #F62A2A`、`confirm #00D872`、`special #904EF0`。
稀有度（低→高）：`#24BAFC`、`#904EF6`、`#FCBA00`、`#FC6C00`。
印刷三原色（注册色条，推断）：`magenta #EC008C`、`cyan #00AEEF`、`yellow #FFF200`。

### 2.2 字体

| 角色 | 字体栈 | Android 回退 |
|---|---|---|
| 正文 `sans` | HarmonyOS Sans SC / Noto Sans SC / Source Han Sans SC | `sans-serif` |
| 展示 `display` | Archivo / Montserrat / HarmonyOS Sans SC | `sans-serif-medium`（或打包 Archivo） |
| 结构/英文 `latin` | Outfit / Urbanist | `sans-serif` |
| 技术 `tech` | Space Grotesk（`tnum` 数字等宽） | `monospace` |
| 等宽 `mono` | JetBrains Mono / Cascadia Code | `monospace` |

> 只列可自由使用的字体；官网商业字体不在其中。Android 端不强制打包字体：系统 `sans-serif` + `monospace` 即可，
> 需要一致观感时再自托管 HarmonyOS Sans SC / JetBrains Mono（注意各自许可）。

### 2.3 字阶（sp，正文 = 16）

`10 / 12 / 14 / 16 / 20 / 24 / 32 / 40 / 52 / 72 / ghost(clamp 80–200)`，
行高倍率：micro 1.0、xs 1.25、sm 1.4、base 1.6、lg 1.2、xl 1.1、≥2xl 1.0；
字距：标签 `+0.05em`、镂空巨字 `-0.04em`。

### 2.4 间距 / 形状 / 阴影 / 动效

- 间距基 4dp：`4 / 8 / 12 / 16 / 24 / 32 / 48 / 64 / 96`。
- 圆角：默认 0；`2 / 4 / 6dp` 三档；胶囊/圆用 `full`。切角：`6 / 10 / 16dp`（`cut-sm/md/lg`）。
- 描边：1dp（暗色下 `line #383838`）；强调 1dp `lineStrong`。
- 阴影：零偏移环境阴影 `0 0 4/8/12/24dp rgba(0,0,0,0.25–0.3)`，不用投影方向光。
- 动效：`50 / 200 / 300 / 400 / 600ms`，`ease` / `ease-in-out` / `ease-out`；进场位移 4–8dp；`prefers-reduced-motion` 等价于 Android `Settings.Global.ANIMATOR_DURATION_SCALE == 0` 时直接到终态。

## 3. 母题（Motifs）

| 母题 | 规格 | 用途 |
|---|---|---|
| `corner-brackets` | 八段短线，臂长 8–16dp，1dp 粗 | 面板四角取景框 |
| `cut` | 单角 6/10/16dp 斜切 | 可操作卡片、页签 |
| `viewfinder` | 圆环 + 十字刻度 + 数字 | 相机/详情面板 |
| `ghost text` | 镂空巨字，`-0.04em` | 页面背景标题 |
| `hatch` | 45° 斜纹，6dp 周期 | 禁用/限时区域 |
| `registration strip` | 品红/青/黄三段色条 | 页脚、卡片顶部 |
| `tick ring` | 刻度圆环 | 进度、倍率、FOV |
| `dashed leader` | 1dp 虚线 + 端点 | 数据行对齐 |

## 4. 组件规格（Android 映射）

| 组件 | 规格 | Android 实现提示 |
|---|---|---|
| Button（action） | 直角，信号黄底，`onAction` 字，16sp/lg，内边距 24×12dp，按下 `actionPressed` | `GradientDrawable` 或 Canvas 矩形；无圆角 |
| Button（control） | `control` 底、`onControl` 字 | 同上 |
| Button（ghost） | 透明底 + 1dp `lineStrong` | 描边 |
| Tag | 直角，12sp，`0.05em`，1dp 边或实底；成对使用时右侧副标 | `TextView` + background |
| Tabs（楔形） | 选中块右缘 45° 斜切 10dp，墨块滑动 200ms | Canvas `Path`，叠加 `ValueAnimator` |
| Panel | `surfaceRaised` + 1dp `line` + 角括号 | 自绘 `Drawable`：8 段线 |
| SectionTitle | 三段入场：滑块左入→箭头转正→文字淡入（各 200ms，错开 60ms） | `ViewPropertyAnimator` |
| ItemSlot | 64dp 方格 + 稀有度底纹 + 左下数量 | 自绘；稀有度只用底，不加圆角 |
| ItemGrid | 8dp 间距，4–6 列自适应 | `GridLayoutManager` |
| Progress | 条高 4dp，信号黄填充；不确定态 40% 宽往返 1.2s | `ValueAnimator` |
| TickRing | 1dp 圆环 + 每 15° 一短刻度；中心数字等宽 | Canvas |
| Dialog | 遮罩 `scrim`，面板分段入场（上/下/左/右各 8dp，300ms） | `Dialog` + Window 动画 |
| Toast/Alert | 左侧 4dp 状态色竖条 + 12sp 英文标签 | 自绘 |
| Toolbar / SideRail | 顶部 1dp 线，英文小标 `0.05em`；侧轨宽 56dp | `LinearLayout` |
| Slider/Stepper | 轨道 2dp，滑块 12dp 方块（不圆） | 自绘 |
| ItemTooltip | 悬浮卡：1dp 边 + 角括号 + 名称/稀有度/描述 | `PopupWindow` |

组件代码不在本版；v0.2 再提供 View/Compose 封装。设置页 `EnhancementSettingsActivity`、`GameOverlay` 的现有控件可直接按上表替换配色与形状。

## 5. 游戏内面板的特别约束（Android）

- 面板显示在游戏之上，**不能用模糊/大阴影**（GPU 成本高且与游戏 HUD 混淆）；只用 1dp 线和半透明底。
- 默认暗色（游戏多为暗场景）：`surface #141414`、`surfaceRaised #1F1F1F`，信号黄保持。
- 安全区：横向 16dp、纵向 24dp；屏占比 > 90% 时缩短为 8dp。
- 触控目标 ≥ 44dp；滑块/步进器加宽命中区。
- 叠加层每帧只重绘脏区；切角/角括号用 `Path` 缓存，避免每帧重建。

## 6. 素材来源与合规（重要）

游戏 UI 的**真实取色/字体/截图**可以用 ForTMR 主仓的 VFS 工具在本机取得，但**不能进仓库**：

```bash
# 在 ForTMR 主仓（EFML4A）里构建纯 C 工具链：6 组 / 33 子命令
cd tools/vfs && make            # 或 make cli
./cli/build/fortmr help

# 解析/抽取（输出只留在本机，勿提交）
./cli/build/fortmr blc info <game.blc>                    # 条目/结构（原 vfsfull）
./cli/build/fortmr mem extract <src> <listfile> <outdir>  # 按 off/len/name 列表并行抽取
./cli/build/fortmr tex dds2png <in.dds> <out.png>         # BC7 → PNG
./cli/build/fortmr tex detect <in.png>                    # 检测主饱和色
```

本仓库只保留**原创 SVG/PNG 预览**与**测量得到的颜色/字号**（数值本身不构成素材复制），
以及 MIT 许可的 [`endfield-ui`](https://github.com/icaruszezen/endfield-ui) 令牌的适配版本。
官方字体、图标、立绘、Logo、界面贴图一律不入库；`ui-kit/NOTICE.md` 记录来源与边界。

## 7. 路线

| 版本 | 内容 |
|---|---|
| v0.1（本版） | 令牌（JSON + Android XML）、组件规格、母题说明、预览图 |
| v0.2 | `ui-kit/android` 提供 `EuxTheme` 工具类、`EuxPanelDrawable`、`EuxButton` 等 View 封装 |
| v0.3 | 设置页 / 游戏内面板按规范重构；Compose 封装（若迁移） |
| 之后 | 与 ForTMR2 的 mod 包内 `[ext.ben.ui]` 主题段联动（可选） |
