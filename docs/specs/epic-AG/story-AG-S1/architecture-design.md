# AG-S1 架构设计

## 决策与依赖

依据AD-OCT-05连接内门禁、AD-DS-01宿主研发准入、AD-ST-01MVP任务库。Architecture Impact：conforming，未变更协议、持久化或状态所有者。

TaskHost拥有已恢复SQLite和唯一Admission；GatewaySession拥有连接内AuthContext/协商标记。正式组合根query_session接收可信Rust会话，调用Application的GatewaySession.handle_encoded；编码沿用Rust协议encode，不由desktop手写JSON。desktop仍只依赖Application/Adapters，Application不依赖TaskHost或具体存储。

UI的LocalUser查询入口继续固定本机身份；Agent不得走此入口。会话身份只由未来完成认证的Adapter/组合根建立，agent_id字段不是凭据。本批库方法不是Tauri命令或网络入口，不接受UI提供身份。

## 调用、失败与验证

可信调用方建立会话→hello→task.get/list/events；请求仍带request_id、agent_id、capability、deadline。相同会话每次重新校验；重连创建新会话。编码/存储不可用返回宿主错误，JSON-RPC错误保留协议编码。不记录正文、完整Payload或查询结果。

实际SQLite双Agent样本验证未握手拒绝、各读所属、越权不可见、重连未握手。已有Application合约覆盖坏格式/版本/过期。本批不创建传输/Sidecar，独立库验证不能代替AC6–7。实际认证与传输设计明确后另做Apply，不通过设备密钥暂缓绕过认证。

## 边界与依赖

Gateway身份/握手归Application会话，TaskHost持唯一数据库；依赖方向不变。

## 状态与契约

协议来自Rust；查询复用既有契约，创建字段/幂等迁移在AG-S2定稿后实施。

## 失败与验证

认证、握手、归属、过期、不可用必须明确失败；真实文件测试与原生连接证据分别保存。

## 正式宿主私有stdio研发接入

依据Accepted AD-AG-03及用户本地Agent测试变更：可信父进程显式启动--local-agent-stdio，固定研发Agent身份，真实TaskHost共享，不另开库。AC1/2/4/5复用Gateway门禁；AC6生产认证仍未满足。Agent仅登记created，已有菜单有任务悬停可见、移入可操作、移出消失；坏帧/EOF终止连接，普通启动不开放入口。Windows暂缓；原生证据与stdio证据分别保存。

## 未开始取消协作

Accepted AD-TM-04由TM-S3首批实现created取消；Agent协议1.2协商task.cancel，用户入口固定LocalUser且仅task-space窗口。详情选择created后提供取消任务按钮，提交中禁用、成功刷新、全部保留取消态、失败提示；不提供执行中取消/接管伪实现，也不新增人工创建。原生证据归tm-s3-pending-cancel，Windows仍暂缓。

## 本地 CLI 与 MCP 增量

依据Accepted AD-AG-05。desktop组合根以`interprocess`和Tokio监听`app_data_dir/agent.sock`，父目录0700、端点0600；首个`gateway.hello`的有效`agent_id`建立该连接专属`AuthContext::Agent`，后续请求由既有Gateway一致性校验拒绝身份切换。完整换行帧上限64KiB，坏帧只关闭当前连接；停止时释放Listener并清理端点。

`apps/yonder-cli`只依赖`crates/protocol`及传输/序列化技术库。`yonder mcp`实现MCP stdio的initialize、tools/list和tools/call，把工具参数构造成Rust协议类型并经共享IPC客户端发送；启动时先hello 1.4。CLI不打开SQLite、不持有TaskHost、不启动Tauri、不接受外部agent_id。Windows仍映射Named Pipe但本轮不实现。

## 云端 Connector 边界

Desktop组合根只持有一个Cloud Connector Adapter。Adapter主动建立WSS，完成外部平台认证后构造固定`AuthContext`与`GatewaySession`，所有帧继续进入现有Application用例；Adapter不直接访问TaskStore、AgentInputHub以外的具体Adapter，也不定义第二份协议模型。

连接状态只驻留内存：`disabled → connecting → authenticated → backoff`。只有`authenticated`且hello成功后才向统一会话注册表注册能力；断线立即撤销会话并使未确认请求归unknown。指数退避必须有上限和抖动，网络恢复不重放未确认副作用。事件补传复用SQLite Outbox与`last_sequence`，输入正文不进入Outbox。

产品端点、配对流程、令牌格式、轮换与撤权由外部平台契约提供。系统TLS验证不可关闭。持久设备凭据只能进入获批的系统Credential Store；当前接线延期时Connector保持disabled。Spike可使用进程内测试凭据和本机隔离服务，退出即清理，不生成产品默认值。

## 快慢脑模式 Gateway 增量（design-review）

沿用既有 `GatewaySession`、`AuthContext`、Rust 协议派生与 TaskHost；不改 Socket/CLI/WSS 传输层，也不引入 Jev 到 Gateway。首个计划与后续 replan 采用同一受版本约束的请求语义，可信会话身份、任务归属、能力、deadline、`request_id`、任务序号与计划版本在 Application 边界校验。Gateway 只路由到 EX-S2 计划用例，不能直接调用具体 Jev Adapter 或 Driver。

跨 Story 调整以 [AD-AG-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-07-SLOW-BRAIN-GATEWAY-INGRESS.md) 为准：AG-S1 既有本地接入与云端 Spike 不返工；AG-S3 只保证归属 Agent 声明继续可用；TM/CU/BU/DO/CM/FI 的状态、Driver、锁和命令围栏先保持现状。快脑内部步骤来源、统一启动与事件事务均等 EX-S2/TM-S2/TM-S7 联审定案后才可生成实施 OpenSpec。

EX-S2/TM-S2/TM-S7 联审计划片段的最小 Rust 类型、幂等与 CAS、SQLite 当前计划事实、事件/Outbox 原子提交及快脑内部步骤来源。失败返回版本/归属/能力/过期/冲突的分类结果，不保存完整计划正文或模型 Payload 到日志。快脑交回使用现有事件序号和 Outbox；`task.events(after_sequence)` 的权限和旧版本投影继续生效。对外新增协议能力须在 AD-AG-07 定案、Story 三份设计与 OpenSpec 完成后实施；旧 Agent 逐步调用不被强制迁移。

验证先用同一 Application/Gateway 合约覆盖本地与云端认证会话的等价请求、跨 Agent 拒绝、版本/CAS、重复请求、断线后按序读取交回依据；随后分别取 Windows Named Pipe/macOS UDS 与云端 Connector 可用时的原生证据。Windows 当前暂缓，产品云端 Connector 仍受 AG-S1 原有配对/凭据门禁。
