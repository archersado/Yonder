# AD-AG-02 Agent任务登记与幂等事务

状态：Accepted（首批本地登记核心与联调；生产连接认证/IPC未获本决定实施许可）。关联AG-S2、TM-S1、DS-S2、AD-AG-01/AD-ST-01。Architecture Impact：architecture-change（协议与持久化扩展）。

用户允许先通过本地Agent连接测试。首批task.create是任务登记，不解释或执行description，不接受Agent自报running/成功；后续执行动作按对应Capability Story实施，原产品步骤/资源/审计需求仍保留。不支持人工创建。

候选协议升级1.1：task.create参数agent_id、capability=task.create、deadline、idempotency_key（沿用1–128 ASCII标识约束）、description（非空、最多4096 UTF-8字节，仅说明，不是Shell/Planner输入）。request_id沿用JSON-RPC id。Agent没有可选owner、status、sequence、task_id字段。创建响应复用kind=snapshot，返回当前task_id/owner/status/sequence；重复请求返回既有任务最新状态，不回滚状态。1.0会话仍可查询，不开放task.create；创建只能从已握手GatewaySession进入，可信本机query入口拒绝。

TaskStore新增幂等登记Port，默认不支持且拒绝写；Sqlite Adapter支持。SQLite schema2→3同事务新增task_creations(owner_agent_id,idempotency_key,task_id,description)，主键为Agent+key；description逐字节匹配，同key不同description返回幂等冲突-32009，同内容返回同任务。任务ID由Adapter在事务内用SQLite randomblob(16)生成task_十六进制，不引入依赖。创建任务/created事件/Outbox/幂等映射同一IMMEDIATE事务；任一失败全部回滚。保留幂等记录，不TTL/自动删除，不为schema1/未知版本迁移。明文与已授权SQLCipher入口复用同一迁移，错误密钥/未知格式拒绝。

记录正文只在受控SQLite内容表，禁止日志输出description/完整请求。有界输入/读帧/队列约束仍保留。权限/版本/过期校验必须在幂等命中返回前完成，不能凭key绕过撤权。

本地联调使用独立测试目录、私有继承stdio管道和预绑定Agent身份的测试宿主，不从请求agent_id构造AuthContext；测试Agent发协议创建，不手工插库。该测试夹具不是产品接入/人类CLI入口，不伪称已接到当前桌面进程，正式UDS/Named Pipe及生产身份认证沿AG-S1继续实施。Windows用户暂缓，不将macOS/库联调扩大为双平台通过。

## 本地任务登记设计定稿（AD-AG-02）

首批创建协议/幂等字段按Accepted AD-AG-02定稿，先做独立本地stdio Agent联调核心。task.create只登记说明和created任务，不执行说明、不接受外部自报running；真实动作执行能力及生产认证IPC不属于首批。

协议1.1、task.create参数与返回字段、schema2→3原子幂等迁移、输入限额与冲突-32009按AD-AG-02；重复返回当前真实快照。同一Agent同key同内容重试不增任务/事件/Outbox，不同内容拒绝，两个Agent key隔离；LocalUser、未握手、1.0、伪造身份、过期均在写库前拒绝。不得把任务登记当执行动画验证。

本地测试Agent通过私有继承stdio管道发hello/create/list/get，测试宿主身份预绑定，SQLite使用独立目录。原生当前桌面进程/生产连接认证未接，不冒充已接通；现有任务面板/小龙仍只消费正式宿主真实状态。核心联调没有人工任务创建按钮。

## Agent命名增量（2026-09-14）

关联AG-S2、TM-S1、DS-S2三份设计与Accepted AD-TM-07名称技术定稿。Architecture Impact：architecture-change（协议1.3、SQLite schema5）。只落实名称，不授权接管目标或Recording。

要求：1.3 Agent创建必须提供合法name；旧版不接收name且响应不增加name字段；新版本创建/get/list/cancel返回同一名称。名称1–256 UTF-8字节、非空、无控制字符，原样保存。相同owner/key仅在description和name完全相同才返回既有最新快照，否则-32009，不产生记录。取消后重试不恢复任务。

迁移前唯一SQLite备份，事务新增tasks.name与task_creations.name；旧NULL不伪造名称，保留所有业务记录。未知格式、已有加密旧版本、备份或DDL失败拒绝升级。卡片与详情用名称/历史缺名反馈，ID仍可查看。验证名称边界、旧新版门禁、原子回滚、备份与数据保留、正式嵌入UI和macOS原生显示；Windows暂缓、完整Story不归档。
