# Proposal：TM-S3 显式 CUA 接管与控制卡

## Why

当前 macOS CUA 把任意真实 HID 输入解释为接管并杀死 Worker，造成任务频繁 `unknown/user-input`；用户要求只由 Yonder 上的明确按钮交回桌面控制，同时在 CUA 执行时直接看到控制状态。

## What Changes

- 删除 CUA Worker 的普通 HID 自动中断。
- 增加仅内存的活动 CUA/显式接管信号，不阻塞动作和任务存储。
- 增加与圈选工具条同屏、同顶部居中位置和同视觉样式的控制条窗口。
- 点击后在当前动作完成并 Observe 后复用既有接管与工作定位链路。
- 将控制条生命周期从单次 CUA RPC 提升为 CUA 任务生命周期，步骤间保持可见，终态或显式接管后清理。

## Impact

影响 `crates/adapters` 的 CUA 监管、`apps/desktop` 的 Gateway/Tauri/UI；不新增外部协议、数据库格式或第二执行栈。依据 Accepted AD-CU-07。Windows 暂缓。
