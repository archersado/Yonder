# Proposal：CU-S4受限前台坐标输入产品接线

关联Story CU-S4与Accepted AD-CU-07。Architecture Impact：architecture-change（协议1.39计划参数、Adapter Worker动作路由）；后续唯一版本升级由`cu-s4-ax-visual-fallback`承接，不改SQLite、不新增Driver或任务状态。

## Why

Spike证明固定trycua 0.25.0已经支持精确窗口`type_text(x,y,text,foreground)`与前台恢复。正式企业微信链路此前拆成不可核实后台点击和无坐标输入，无法安全推进。

## What Changes

- 协议1.39为两个受保护文本语义开放恰好`x/y`参数。
- Worker瞬时展开引用并注入精确窗口、受监管session和foreground；Agent不能控制delivery mode。
- 视觉坐标点击使用同一动作级foreground路由。
- 隔离测试覆盖参数门禁、SDK参数、confirmed/Observe推进和unverifiable交回。
- 正式Yonder Gateway重新运行企业微信无发送样本；发送仍需本机一次性确认。

## Non-goals

本Change本身不升级trycua；后续Accepted AD-CU-07修订与独立Apply已改变固定版本。不允许自由文本/键盘宏/桌面全局坐标，不自动重试，不修改Windows范围。
