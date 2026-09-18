# 提案：修复“进行中”任务分页遗漏

Story：DS-S2。关联设计：`docs/specs/epic-DS/story-DS-S2/`。决策：AD-DS-03。

问题：桌面先分页未结束任务再过滤 `running`，非运行任务可占满当前页，使真实运行任务不可见。

变更：为 `task.list` 1.16 增加向后兼容的 `running_only`，由 SQLite 在分页前过滤；桌面进行中视图使用该参数。状态、授权、排序和游标仍由现有 Application/TaskStore 链路负责。

Architecture Impact：conforming。协议边界按 AD-DS-03 更新，不新增状态所有者、存储表或执行能力。
