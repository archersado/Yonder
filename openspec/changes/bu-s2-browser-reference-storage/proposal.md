# Proposal：Browser Task Space 引用持久化

Story：BU-S2。把成功ego-lite Bridge结果与attempt observed、任务事件和Outbox同事务写入SQLite当前引用表，支持重启后读取；不开放Gateway动作。
