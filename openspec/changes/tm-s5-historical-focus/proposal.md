# TM-S5 接管定位历史事实

Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」，TM5-AC02/07/08及接管定位用户变更。决策：Accepted AD-TM-19。

把 Yonder 已提交的定位开始与最终成功/失败保存为不可变历史，避免当前投影覆盖中间阶段。Architecture Impact：architecture-change（SQLite schema 19、协议 1.23）；不扩展原生定位能力，不启用 Recording 或交回。
