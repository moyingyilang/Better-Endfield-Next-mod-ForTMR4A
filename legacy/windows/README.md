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

## P1 追加归档（2026-11-24，Android-only 清退）

| 路径（相对 `legacy/windows/`） | 原位置 | 说明 |
|---|---|---|
| `native/shared/host/` | `native/shared/host/` | Windows Host 实现：bootstrap / host_runtime / module_manager / hook_broker / hook_diagnostics / logging / settings_store / dynamic_resolver；仅保留 `pose_lease.cpp`、`type_name_contract.h` 在主线 |
| `native/shared/include/BetterEndfieldNext/BootstrapConfig.h`、`VoiceCatalog.h` | 同左 | 只被上述 Host 与 Windows 语音模块引用 |
| `native/modules/ui/touch_input.cpp`、`touch_pointer_policy.cpp` | 同左 | Windows 鼠标→触摸实现；Android 由 `android_compat/touch_input_android.cpp` 提供，头文件仍在主线 |
| `native/modules/media_screen/` | 同左 | `capture_broker`、`window_capture`、`remote_capture`、`game_input_*`（D3D11/DXGI/DirectInput）；`module.cpp` 与平台无关头文件留在主线 |
| `native/modules/camera/overlay/`、`native/modules/custom_model/overlay/` | 同左 | GDI+ 悬浮层（`main.cpp`、`.rc`、manifest） |
| `native/modules/custom_model/driver_audio_win.cpp`、`native_mesh_layout.cpp`、`tests/model_overlay_tests.cpp` | 同左 | WASAPI 音频、Windows 网格布局、悬浮层测试 |
| `native/modules/actions/tests/`、`native/modules/weapon_appearance/tests/`、`native/modules/camera/eiem/compat/tests/` | 同左 | 原由 Windows CTest 驱动的测试与夹具 |

主线 `native/modules/custom_model/CMakeLists.txt` 同时移除了引用 `native/tests/` 的 6 个测试 target
（`BemValidate`、`BemV11CapacityTests`、`BemV13MorphTests`、`AnimationDriverTests`、
`BemV14ResourceTests`、`BemKeepTextureValidate`），`BemCompose` 指向归档路径。
