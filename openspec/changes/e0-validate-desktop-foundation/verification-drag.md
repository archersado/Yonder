# 独立 Verification Goal：E0-S1 桌宠拖动补充

日期：2026-09-11。关联：E0-S1、同 Change 的「拖动桌宠窗口」场景、AD-E0-01（仍未通过双平台门禁）。

## 验证目标

1. `node spikes/desktop-foundation/check-drag.mjs` 通过，证明声明了拖动区域且权限仅限本地 `pet` 窗口。
2. Windows Release 离线锁定构建成功，实际启动具有窗口句柄且响应正常。
3. Windows 人工按住紫色区域左键拖动，确认位置改变、松开停止且文字未被选中；记录结果。
4. macOS 同场景验证（按用户要求暂缓）。

## 当前证据

- 配置回归检查通过；此检查不能证明原生拖动已生效。
- Windows 构建与启动：`cargo build --release --offline --locked` 退出 0，耗时 44.47 秒；启动 PID 14132，`HasExited=False`、`Responding=True`、`MainWindowHandle=857886`。使用现有 Windows E0 构建目录，源码与仓库一致。
- Windows 原生拖动：待人工验证。
- macOS：暂缓，无证据。

结论：未通过完整 Goal，不归档、不标记 E0-S1 完成。验证失败回到实施阶段。
