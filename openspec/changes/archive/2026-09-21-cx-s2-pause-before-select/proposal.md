# Proposal：CX-S2 圈选前暂停桌面任务

关联Story：CX-S2；关联决策：Accepted AD-CX-01、AD-TM-08。

## Why

当前圈选入口检测到CUA桌面租约后只返回拒绝，尚未满足CX2-07“用户输入立即暂停且不与Agent争夺指针”。

## What Changes

Desktop可信宿主取得唯一桌面租约所属任务，复用既有TM-S3 `pause`控制与步骤边界停止；只有paused事实提交并释放租约后才显示圈选层。失败或unknown保持隐藏并给出稳定反馈。托盘和小龙入口使用同一用例。

Architecture Impact：conforming。复用Application Admission、TaskStore和Control事务，不增加外部协议、schema、状态所有者、轮询或新依赖。
