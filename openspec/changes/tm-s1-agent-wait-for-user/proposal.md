# 提案：Agent 提交等待用户状态

Story：TM-S1。来源：TM1-AC14、产品简报“等待用户介入”、架构主干等待原因持久化。决策：AD-TM-10。

问题：正式 Gateway 无法把已到安全步骤边界的运行任务切换为 `waiting-for-user`，桌宠已有状态只能由内部测试触发。

变更：增加协议 1.17 `task.wait_for_user`，原子保存等待状态、原因、事件和 Outbox，并在提交后释放准入资源；事件查询向 1.17 会话返回原因。

Architecture Impact：architecture-change（协议与 SQLite schema 13→14）。不实现 Resume、用户回答投递或等待外部响应。
