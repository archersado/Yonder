# 设计

Application定义Pause/Cancel/Takeover控制种类及停止记录。Store在单事务内核对running任务、完整observed attempt与当前sequence，提交paused/cancelled迁移、事件、Outbox和attempt stopped。相同控制幂等，不同控制/旧身份冲突。Permit先调用停止用例，提交成功后释放；失败把Permit交还。
