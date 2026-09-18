# 独立Verification Goal：Agent任务名称

状态：PASS（核心与macOS名称子范围；Windows暂缓）；名称子范围，完整Story不Done/Archive。环境2026-09-14 macOS，Windows用户暂缓。关联AG-S2 NAME-01～04、TM-S1 AC-10、DS-S2名称显示，Accepted AD-TM-07、ag-s2-local-task-registration名称增量。

验证目标：新协议Agent名称贯穿真实创建/get/list/cancel与原生菜单；旧版严格快照兼容；幂等冲突/取消后重试不改变记录；SQLite备份迁移保留历史说明/状态/事件/Outbox，DDL失败回滚。UI只读Rust同源状态，无人工创建/本地模型摘要/数据删除。

核心阶段：cargo test --workspace --offline --locked，31项通过；其中新增名称版本/幂等/限额合约与迁移备份/历史/失败回滚两项。架构关联与生成协议检查通过。已有ts-rs deny_unknown_fields解析提示不影响Serde严格校验；不称新增警告为失败。

原生阶段待完成：编译正式Debug宿主，先退出旧唯一实例，私有stdio预绑定测试Agent协商1.3真实创建两项具名测试任务；hover菜单/详情显示名称及ID、面板移入可操作/移出隐藏，截图与结构化布尔日志。测试后通过task.cancel取消，保留数据，EOF不关闭桌宠。已有两项Cancelled任务不得改名或复活。迁移前基线schema3/tasks2/cancelled2/events4/outbox4/creations2。

失败返回实施阶段，不修改运行中任务或伪造结果；接管停止/目标屏幕与Recording不属本Goal。

## 实测结果

本地stdio Agent真实创建两项具名任务、重试返回同快照；原生宿主PID66662。菜单hover/移入操作/移出隐藏、卡片名称、具名详情与任务ID、刷新与同一可见桌面全部通过。截图已目视检查，两项Agent名称清晰显示；详情区域沿既有轻量面板滚动。测试后task.cancel成功，两项记录保留，小龙EOF后继续运行。

证据：[Agent](../../../apps/desktop/evidence/agent-names-20260914/agent-result.json)、[原生](../../../apps/desktop/evidence/agent-names-20260914/native/result.json)、[截图](../../../apps/desktop/evidence/agent-names-20260914/native/native-agent-tasks.png)、[迁移](../../../apps/desktop/evidence/agent-names-20260914/migration-result.json)。schema5/tasks4/cancelled4/named2/events8/outbox8/creations4；SQLite备份仍schema3，逐列比对所有旧任务/事件/Outbox/创建记录原样存在，无复活/删数据。31项Workspace测试与架构关联/生成协议/差异格式检查通过。

范围结论：NAME-01～04及菜单名称子范围通过。真实任务执行、接管停止与工作定位、生产认证/IPC及Windows验证仍不借本结果通过；完整AG-S2/TM-S1/DS-S2不Done/Archive。
