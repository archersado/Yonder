# AG-S2 Agent任务创建接入

Story: AG-S2
Epic: AG
Status: implementing
OpenSpec: ag-s2-local-task-registration

设计：[产品需求](product-requirements.md)、[架构设计](architecture-design.md)、[视觉交互设计](visual-interaction-design.md)。依据Accepted AD-AG-01仅Agent创建；首批登记协议和幂等事务已按Accepted AD-AG-02定稿并实施。生产认证、正式宿主IPC和执行派发仍保留设计门禁。

2026-09-14首批权限门禁设计审阅通过，只实施Application拒绝LocalUser创建；其余设计仍联审。Proposal：openspec/changes/ag-s2-agent-create-guard/，不开放外部创建。

首批独立verification-create-guard.md：19项联合回归与架构门禁通过。Application禁止人工创建已落实；尚无真实Gateway创建入口，完整Story不Done/Archive。

首批登记字段设计审阅通过，Accepted AD-AG-02；后续实施目录openspec/changes/ag-s2-local-task-registration/。先前创建权限变更保留历史，不Archive完整Story。

本地登记核心与私有stdio测试Agent联调通过，独立验证见[Verification Goal](../../../../openspec/changes/ag-s2-local-task-registration/verification-local-agent.md)。两个任务由Agent协议调用真实落库；未接到当前桌面进程。当前Change承接前次权限防线，前次Change只保留子范围历史。完整Story仍implementing，不Archive。

2026-09-14 Agent名称子范围已实施并通过31项核心回归和macOS原生stdio Agent菜单验证，见openspec/changes/ag-s2-local-task-registration/verification-agent-names.md。旧数据备份迁移保留，新测试任务取消后保留；完整Story状态不变，Windows暂缓。

2026-09-21 macOS生产UDS与MCP登记联调通过，见[Verification Goal](../../../../openspec/changes/ag-s2-local-task-registration/verification-production-local.md)。首帧Agent身份、幂等、归属隔离、创建事件同事务和取消保留已验证；Windows Named Pipe与原生桌宠端到端仍未通过，完整Story保持implementing。

2026-09-21 macOS原生桌宠面板端到端通过，见[Verification Goal](../../../../openspec/changes/ag-s2-local-task-registration/verification-native-panel-macos.md)。生产UDS登记的两个Agent任务在原生悬停面板可见，详情、按钮可用性和失焦隐藏均已验证；Windows仍暂缓，完整Story保持implementing。
