# CX-S2 macOS Preview 验证目标

状态：进行中  
范围：仅 macOS 当前显示器的圈选提问 Preview。

## 通过条件

1. 从小龙悬停图标或菜单栏入口打开一次短生命选择层；未启动时不存在覆盖层。
2. 成功圈选后选择层退出，选区旁确认卡显示内存缩略图；不创建任务、不发送 Agent 输入、不保存截图。
3. Esc、关闭、30 秒超时、权限拒绝和 CUA 桌面租约均清场且不留下预览数据或任务事件。

## 已有检查

- `cargo check -p yonder-desktop` 通过。
- `python3 scripts/check_architecture.py` 与 `git diff --check` 通过。
- `cargo test -p yonder-application admission::tests --lib` 通过。

## 待实机验证

首次圈选按系统提示授予 Yonda 屏幕录制权限，分别验证成功圈选与 Esc 取消。证据只记录通过条件，不保存桌面截图本体。
