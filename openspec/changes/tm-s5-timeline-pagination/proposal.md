# 提案：既有任务时间线分页

Story：TM-S5。来源：TM5-AC08、TL-04。决策：AD-TM-09。

问题：Task Space 只读取首批 100 条事件，已有 `after_sequence` 能力未提供继续查看入口。

变更：复用现有 `task.events` 排他序号游标，在详情中显式加载并按序追加后续页；局部错误保留已有时间线，迟到响应受任务选择轮次保护。

Architecture Impact：conforming。无协议、持久化、依赖或状态所有者变化；产物、确认、新 payload 字节预算和清理不在本 Change。
