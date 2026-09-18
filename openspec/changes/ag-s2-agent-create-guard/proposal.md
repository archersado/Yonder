# AG-S2 Agent专属创建权限门禁

关联AG-S2三份设计、docs/specs/epic-AG/story-AG-S2/及Accepted AD-AG-01。现有Application create仍接受LocalUser，与用户不支持人工创建不符。共享创建入口在存储前拒绝本机用户，只允许可信Agent上下文。

Architecture Impact：conforming；新增内部PermissionDenied错误沿用既有-32003响应形状，无协议字段/SQLite迁移/依赖变化。此子范围不新增task.create方法、认证Adapter或演示任务。字段级创建/幂等仍待设计，不扩大授权Apply。
