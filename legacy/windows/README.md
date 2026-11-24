# legacy/windows

Windows 构建链的**归档区**。这里的内容自 ForTMR4A 平台收敛（2026-11-24）起不再参与任何构建，
只作为审计、许可追溯和未来移植参考保留。**不要在此目录内继续开发**。

## 归档内容

| 路径 | 原位置 | 规模（非空行） | 说明 |
|---|---|---:|---|
| `ui/` | `ui/` | 25,338 | WinUI 3 桌面管理界面（C# + XAML），含 `ui/tests` |
| `installer/` | `installer/` | 447 | Inno Setup 安装器（`.iss` + 中文语言文件） |
| `native/loaders/` | `native/loaders/` | 1,533 | Windows 注入器、官方启动器校验、XInput 代理 |
| `native/modules/combat_stats/` | 同左 | ~10,700 | Windows 战斗统计模块与 GDI+ 悬浮层 |
| `native/modules/gacha/` | 同左 | ~460 | 寻访记录导出模块（命名管道） |
| `native/modules/music/` | 同左 | ~3,000 | Windows 本地音乐 / OmniMix 模块 |
| `native/modules/voice/` | 同左 | ~3,200 | Windows 语音路由模块（Android 侧为 `character_voice`） |
| `native/CMakeLists.txt` | `native/CMakeLists.txt` | 420 | Windows 主机侧 CMake 入口（46 个 target） |
| `native/tests/` | `native/tests/` | — | Windows CTest / 自研 CHECK / JVM 桩测试 |
| `native/research/` | `native/research/` | — | `music_probe`、`touch_probe` 等 Windows 探针 |
| `native/tools/` | `native/tools/` | — | `bem_compose_main.cpp` 等 Windows 开发工具 |
| `scripts/*.ps1` | `scripts/` | ~2,000 | Windows 构建、签名、混淆、安装器脚本 |
| `scripts/ci/windows.ps1` | `scripts/ci/` | — | Windows CI 构建脚本 |
| `.github/workflows/release.yml` | 同左 | 445 | 原四段发布流水线（Windows / Android / tools / publish） |
| `BetterEndfieldNext.slnx`、`global.json` | 仓库根 | — | 已无 .NET 构建，仅作记录 |

## 注意

- 文件是**整体移动**而非复制；内部相对路径仍按原 `native/`、`scripts/` 布局书写，
  因此**不能当作可构建工程直接使用**。
- 第三方来源与许可（EIEM 快照、MinHook、7-Zip、Inno Setup 等）见仓库根
  `LICENSE`、`THIRD_PARTY_NOTICES.md` 与对应目录内的 LICENSE/NOTICE 文件。
- 若未来恢复 Windows 支持，应以 ForTMR2 的 CRX 边界重新实现，而不是把此目录搬回主线。
