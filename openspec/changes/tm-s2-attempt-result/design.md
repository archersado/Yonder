# 设计

Application 定义 `AttemptConclusion` 与结果记录；TaskStore 原子提交当前完整 attempt 的 `prepared→observed|unknown`。Adapter CAS running 任务 sequence，追加 `running→running` 事件及 Outbox，再更新 attempt 的结果字段；任一步失败全部回滚。相同结论幂等，不同结论/旧身份冲突。协议1.5的 TaskEvent 增加可选 attempt_result，旧版本剥离。
