# AG-S1 统一任务读取与 Gateway 握手

Story: AG-S1
Epic: AG
Status: verifying
OpenSpec: ag-s1-local-cli-mcp

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

认证与宿主未接线，密钥暂停；现有核心成果在旧 OCT-S1 Change，后续另建本 Story Proposal。

与 TM 的边界：AG 向可信用例提供发起者和授权引用，并在后续调用重新校验有效授权；TM-S1 的授权历史或归属快照不能替代当前权限。任务最新详情归 TM-S1、历史/产物归 TM-S5，派生协议须共同审阅，不能让任意请求自报可信作者。

## OpenSpec 与验证

尚未生成；三份设计明确后再创建。

2026-09-14三份设计完成首批库接线审阅；关联独立ag-s1-host-gateway-query。只做可信会话到正式TaskHost查询，传输认证与真实任务提交未开放，不Done。

实施提案：[ag-s1-host-gateway-query](../../../../openspec/changes/ag-s1-host-gateway-query/proposal.md)；关联目录openspec/changes/ag-s1-host-gateway-query/。

当前增量openspec/changes/ag-s1-desktop-private-stdio/，Accepted AD-AG-03定稿后实施；之前宿主查询Change保留核心验证历史。

正式小龙私有stdio研发接入及原生菜单验证PASS，独立记录[Verification Goal](../../../../openspec/changes/ag-s1-desktop-private-stdio/verification-desktop-agent.md)。固定研发Agent登记两项created任务，生产Socket认证与Windows尚未通过，完整Story不Done/Archive。

2026-09-15用户确认由安装包内`yonder` CLI承接Codex接入，正在实施[ag-s1-local-cli-mcp](../../../../openspec/changes/ag-s1-local-cli-mcp/proposal.md)：桌面进程持有当前用户私有UDS，`yonder mcp`以stdio适配MCP并复用统一Gateway；不创建第二个App或第二份任务状态。

2026-09-18补齐云端Connector设计：Yonder只建立单一出站WSS，并把认证结果绑定为同一`AgentSession`；AG-S5只复用该会话。外部平台配对协议、端点配置和持久设备凭据尚未给定，且系统Credential Store接线按用户决定延期，因此当前不生成产品WSS Proposal、不使用匿名连接、硬编码令牌或明文凭据。

技术路线进入[ag-s1-cloud-connector-spike](../../../../openspec/changes/ag-s1-cloud-connector-spike/proposal.md)：只验证Rust进程内WSS、系统TLS、断线释放和有界重连。产品Connector仍受外部平台契约与凭据门禁阻塞。

2026-09-18 macOS独立Verification Goal通过：`tokio-tungstenite + rustls`单进程样本完成可信WSS、Ping/Pong、关闭后重连、64 KiB帧上限与无效TLS拒绝。Windows按用户决定暂缓，AD-AG-06仍为Proposed，产品Connector未接线。

2026-09-20用户变更：本地UDS不再固定`codex-cli`身份。新增[连接身份绑定 Change](../../../../openspec/changes/ag-s1-session-agent-binding/proposal.md)：首个`gateway.hello`绑定连接，后续请求不得切换身份；CLI必须显式提供`YONDER_AGENT_ID`。

## 新架构：慢脑计划 Gateway 入口

2026-09-21 按 Accepted AD-EX-01 与用户明确的链路要求，首次计划和 Observe 后 replan 仍经 AG-S1 现有 Agent Gateway 入站；快脑交回依据沿任务事件/Outbox 到归属 Agent，不建立直连或第二通道。接入边界见 Accepted [AD-AG-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-07-SLOW-BRAIN-GATEWAY-INGRESS.md)，未决计划契约见 [EX-S2](../../epic-EX/story-EX-S2/README.md)。本增量处于 design-review：计划载荷、版本/CAS、能力协商和事务尚未与 EX/TM 定稿，不生成实施 OpenSpec 或修改协议代码；AG-S1 既有本地身份绑定实现与验证状态不因此重做。

2026-09-23：已补充计划入口联审候选，见[架构设计](architecture-design.md)；该候选与 EX-S2/TM-S7 的计划片段和启动事务边界对齐，仍不授权实施。
