# 上游同步流程

上游：`Dr-hydra/Better-Endfield`（BEN 官方）。本分支是 mod 分支，**必须持续跟上游**。

## 原则

- 每次上游发版（release/tag 或 main 有新提交）都同步一次；
- 用 **merge**（不要 rebase 上游历史），保留上游 commit 与 release notes；
- 冲突处理优先级：
  1. 上游行为/资源/协议变更 → **必须接受**；
  2. 本分支结构（Android-only、`legacy/windows/` 归档、CRX 实现）→ 保留结构，把上游 Windows 侧变更落到 `legacy/windows/`；
  3. 两边都改的 Android 文件 → 以“上游行为 + 我们的扩展”合并，合并后必须跑验收；
- 同步后必须：`assembleRelease` 出包、`efml validate`、真机导入冒烟（有设备时）；
- 同步完成前不要开新的重构分支，避免叠加冲突。

## 操作

```bash
# 1. 看上游有什么新东西
scripts/sync-upstream.sh check

# 2. 合并（新分支上操作，便于回退）
git checkout -b sync/upstream-$(date +%m%d)
scripts/sync-upstream.sh merge

# 3. 解决冲突 → 构建 → 验收 → 合并回主分支
git commit
git checkout better-endfield-next-mod-ForTMR4A
git merge --ff-only sync/upstream-$(date +%m%d)
git push origin better-endfield-next-mod-ForTMR4A
```

## 上游资源包

Android 核心资源包在 releases 的 `game-resources` tag：

```bash
# 按 config/resource-packs.json + android/resources/resource-source.json 算出名字
gh release download game-resources --repo Dr-hydra/Better-Endfield \
  --pattern 'BE-Core-Android-<gameVersion>.<revision>.zip' --dir build/ci
```

解出 `model/character-presets.json`、`voice/voice-catalog-index.json` 到 `android/resources/`，
与 `scripts/ci/fetch-core.ps1` 的映射保持一致。同步上游后版本号变了就必须重取。

## 记录

每次同步在 `docs/workspace/UPSTREAM-SYNC.md` 末尾追加一行：日期、上游 HEAD、本分支 merge commit、
跳过的上游变更及原因（不应有“跳过”，只允许“归档到 legacy”或“后续单独迁移”）。

## 同步记录

| 日期 | 上游 HEAD | 本分支 merge | 说明 |
|---|---|---|---|
| 2026-11-26 | `59d6e4a` (4.2.5) | 本次 merge | 首次完整并入 4.2.2–4.2.5：纹理设备能力、资源 1.7.2.2、相机描边、Ogg Vorbis、mod-center、Java 11 修复；Windows 变更落到 legacy/ |
