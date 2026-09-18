当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：E0-S1 透明小龙

日期：2026-09-11。关联 E0-S1、AD-E0-01 透明窗口补充、同 Change 的透明小龙场景。

状态：实施及 macOS 编译/启动通过，完整视觉与双平台验证未通过。

## 已取得证据

- PNG 为 1254×1254，`sips -g hasAlpha` 返回 yes；静态前端只展示该图片。
- macOS 开启 `macOSPrivateApi` 与对应 Cargo feature，页面背景透明，窗口无边框、无阴影；保留图片原生拖动区域及最小权限。
- 首次透明版本 Release 锁定离线编译通过，耗时 17.14 秒，启动 PID 49591。
- 用户反馈原尺寸偏大后，将窗口从 280×280 缩到 200×200；再次编译通过，耗时 11.39 秒。终止旧实例后启动新实例，PID 50100 存在。
- 环境：macOS 26.5.1，Apple Silicon，Rust/Cargo 1.98.1，Tauri 2.11.5。

## 限制与待验证

- CoreGraphics 按 PID 查询的可见窗口列表为空，不能据此验证实际尺寸、透明或原生拖动；也不能仅凭进程存在宣称视觉验收通过。
- 尚无截图/视频，Windows 本批未构建或运行。Goal 保持未通过，不能 Archive。
- Node 不在 PATH，既有 `check-drag.mjs` 本批未运行；未为此安装新的运行时。
