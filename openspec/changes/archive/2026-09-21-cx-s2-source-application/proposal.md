# Proposal：CX-S2 来源应用显示

关联Story：CX-S2；关联决策：Accepted AD-CX-01。

## Why

当前确认卡只显示截图和问题，缺少产品需求中“提交前展示来源应用”的可见边界，用户无法确认截图来自哪个工作应用。

## What Changes

macOS Desktop在显式启动圈选、显示覆盖层前读取一次系统前台应用名称，交给PreviewSession临时持有并显示在确认卡。重新圈选保留原来源，会话结束清零；不可用时显示“当前桌面”。来源不发送、不持久化、不持续监听。Windows继续按用户决定暂缓。

Architecture Impact：conforming；不新增协议、数据库、依赖或权限。
