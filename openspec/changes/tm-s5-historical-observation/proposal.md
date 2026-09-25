# TM-S5 历史 Observe 事实

Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」及 TM5-AC01/03。决策：Accepted AD-TM-17。

把已有可信 Observe 写入事实按事件序号投影到 `task.events`，使多步骤历史可检查、unknown 不被最新快照覆盖。Task Space 显示与动作结论分离的 Observe 文案。

Architecture Impact：architecture-change。Rust 协议 1.21 增加可选 `TaskEvent.observation`，TypeScript/JSON Schema 从 Rust 生成；不新增表、迁移、写入者、依赖或自动采集。产物版本、迟到观察写入、总配额及清理不属于本 Change。
