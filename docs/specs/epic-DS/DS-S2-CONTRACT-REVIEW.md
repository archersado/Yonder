# DS-S2 查询契约联审

日期：2026-09-14。依据DS-S2三份设计、产品简报「Task Space与权限模型」、AD-OCT-02/03、AD-TM-02与现有Rust契约。状态：完成现状联审；尚不具备完整实施准入。

## 已确认契约

`crates/protocol/src/lib.rs`为唯一协议源。task.list有after_task_id、include_finished、limit（1–100）；响应为tasks与next_after_task_id。task.get返回相同TaskSnapshot字段：task_id、owner_agent_id、status、sequence。sequence为字符串，只能逐任务比较。名称、能力类别、步骤、等待/错误原因、外部浏览器引用均未提供，不能从ID推断或自行补协议。

include_finished=false返回created/running/waiting-for-user/paused/interrupted；true返回全部，不能把true页面标成“仅历史”。首批设计调整为“进行中/全部”，历史单独筛选仍由后续TM/协议设计承接。分页按任务ID，列表不是固定时间快照；刷新从第一页开始、替换旧页，不能跨不同刷新拼接成一致快照。任务消失/终态变化后重新查询，不重放事件来创造当前状态。

Application的AuthContext由可信调用方构造，LocalUser允许本机任务总览，Agent按归属过滤；请求agent_id必须匹配已绑定身份。UI不得传入LocalUser或自报可信身份。已有GatewaySession仅进程内核心，不等同于真实认证及桌面接线。

## 首批展示设计

复用Rust生成类型，只读列表、分页、刷新、选中详情；行使用真实task_id、owner_agent_id、status，详情显示sequence。缺失名称用ID，缺失原因/来源/步骤明确标为未提供；没有外部引用时浏览器入口明确不可用。控制动作仍依TM-S3/S4，保留原始AC，不因首批缺字段缩减完整Story。

进行中/全部均显示授权范围内任务，不把UI页数当忙碌事实源。读取失败保留旧页并标记过期，初次失败明确读取失败；能力未接通显示能力未提供，不能显示“暂无任务”。普通列表刷新不触发任务状态迁移。

## 实施前必须解决

1. AD-E0-01与AD-TM-02的桌面阶段准入存在已记录循环。性能放宽不自动授予正式宿主接线。须先形成阶段决策，区分Spike基础准入、macOS研发接线与完整双平台完成；保留Windows暂停与最终证据。
2. 确定可信本机身份、真实SQLCipher连接、恢复完成及唯一准入实例的桌面组合根；不得使用UI输入身份、明文数据库或假任务数组代替。密钥工作继续暂停，不能借总览自动恢复密钥实施。
3. 环绕菜单仍暂停；面板正式入口未定，不能顺带实现DS-S3。只读面板不得替代所有控制、历史和外部Task Space的原始验收。

本次不生成授权Apply的Proposal，不修改协议/持久化或产品代码。下一项工作为上述阶段准入架构决策；解决后再将已明确首批范围转成OpenSpec，未解决字段保持依赖待办。

已形成具体待审方案[AD-DS-01](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-DS-01-DEVELOPMENT-STAGE-ADMISSION.md)。现有真实SQLCipher合约回归`task_list_pages_committed_states_without_duplicates_or_writes`与`task_ownership_applies_to_every_query_and_preserves_legacy_data`均通过；只证明核心分页与授权契约，不等同于桌面真实接线。架构关联及diff检查通过。
