# 版本号规则

Android 版本名（`versionName`）采用 **语义化版本 + 构建元数据**：

```
<MAJOR>.<MINOR>.<PATCH>+ts.<yyyyMMdd'T'HHmmss>UTCp0800.sha.<shortSha>.build.<buildNo>
```

示例：

```
4.2.5+ts.20261126T231746UTCp0800.sha.a48e821.build.40205
```

| 段 | 来源 | 说明 |
|---|---|---|
| `MAJOR.MINOR.PATCH` | `Directory.Build.props` 的 `<Version>` | 跟随上游 BEN；本分支不改语义版本 |
| `ts.<yyyyMMdd'T'HHmmss>` | 构建时刻 | 时区固定 **UTC+08:00**（Asia/Shanghai），无夏令时 |
| `UTCp0800` | 字面量 | `UTC+0800` 的 semver 安全写法（build metadata 不允许 `+`） |
| `sha.<shortSha>` | `git rev-parse --short=7 HEAD` | 无 git 时回退 `BE_BUILD_SHA` / `GITHUB_SHA`，再退回 `nogit` |
| `build.<buildNo>` | Android `versionCode` | 当前规则：`MAJOR*10000 + MINOR*100 + PATCH`（4.2.5 → 40205） |

语义版本段只用于排序/兼容声明，构建元数据不参与排序；Android 升级顺序仍看 `versionCode`。

## 实现

- 生成位置：`android/app/build.gradle.kts` 的 `beVersionName`；
- 每次 Gradle 配置时取时间、git sha；因此**同一提交的两次构建版本名不同**，这是预期行为；
- 源构件版本（tag、release 名）仍使用 `Directory.Build.props` 的纯 `MAJOR.MINOR.PATCH`，
  避免 CI tag 与构建元数据耦合；
- APK 内部 `BuildConfig.VERSION_NAME` 与设置/关于页展示的即完整字符串；
- 归档文件名（`archiveAndroidRelease`）使用完整版本名，目录名同理。

## 可复现构建

需要固定版本名时设置 `BE_BUILD_SHA`，并把时间固定为发布时刻；如需完全可复现，
后续可引入 `SOURCE_DATE_EPOCH` 覆盖 `beBuildStamp`。
