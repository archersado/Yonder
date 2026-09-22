# DS-S2 统一任务空间

Story: DS-S2
Epic: DS
Status: implementing
OpenSpec: ds-s2-task-overview

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-09-18：首批任务总览的 macOS 实施与独立验证已 PASS，覆盖可信本机宿主、真实两任务、轻量菜单、分页/详情/错误保留及并发刷新。详见 [汇总 Verification Goal](../../../../openspec/changes/ds-s2-task-overview/verification-goal.md)。卡片操作由增量 Change `ds-s2-task-card-actions` 承接。浏览器任务的外部 ego-lite 引用读取与用户交接已由 [BU-S2 独立验证](../../../../openspec/changes/bu-s2-browser-reference-read/verification-goal.md)完成。Windows 依用户决定暂缓；执行中接管/停止确认与跨平台证据未完成，Story 保持 implementing。

2026-09-14按AD-DS-01/AD-ST-01工程审阅通过宿主核心独立子范围：标准文件锁、真实SQLite、显式恢复和可信本机只读查询。仅此子范围进入Apply；先前入口门禁仍适用于GUI，完整Story未就绪，不顺带实现环绕菜单或执行器。

[2026-09-14查询契约联审](../DS-S2-CONTRACT-REVIEW.md)已完成：首批复用四个已有快照字段；进行中/全部与分页语义已明确。正式宿主准入、可信身份与真实数据库接线未解决，保持draft，不生成Apply授权；下一步先处理阶段准入架构决策。

2026-09-14开始下一项研发设计：三份设计新增AC1–7来源映射、真实快照/授权边界与轻量面板交互建议。保持draft；入口、查询契约及桌面阶段准入尚需联审，尚未生成Proposal或开始产品代码。性能优化已按用户变更后置，不改变其他依赖门禁。

先完成查询协议和桌面前置门禁，禁止以假任务列表替代执行闭环。

信息依赖：TM-S1 提供当前详情/等待/恢复，TM-S5 提供时间线、产物与用户结果确认，AG-S1 负责当前有效授权。DS 不实现第二状态或审计存储；布局与字段须联审后进入 Proposal。

## OpenSpec 与验证

最新入口已按用户变更改为小龙轻量任务menu，迁入正式宿主并停止旧预览；[菜单独立验证](../../../../openspec/changes/ds-s2-task-overview/verification-menu-macos.md)记录本机真实空库、AX入口/关闭/托盘恢复与21项核心回归PASS。GUI首批进入实施，原有“入口未决/GUI未接入”段落保留历史；完整真实任务与双平台Story仍未完成。

宿主核心子范围已实施，[独立验证](../../../../openspec/changes/ds-s2-task-overview/verification-host-core.md)本机PASS：真实SQLite恢复、跨进程锁、固定本机授权及失败清理通过，三模块联合回归19项通过。GUI尚未接入，现有预览小龙未替换，不宣称总览已可见。

2026-09-14已建立[首批只读展示Proposal](../../../../openspec/changes/ds-s2-task-overview/proposal.md)。AD-DS-01阶段依赖审阅通过；实际可信源与入口仍待满足，Proposal不授权Apply，Story保持draft。原始完整AC及Windows证据未删除。

已生成首批Proposal；入口与可信数据源门禁尚未满足，尚未进入Apply。

2026-09-14用户变更：菜单改由清醒小龙悬停触发，点击仅动作反馈，拖动不打开，双眼点击只唤醒。本机独立verification-hover-macos.md已PASS，Windows暂缓、完整Story仍实施中。

2026-09-23用户变更：清醒小龙右键可手动唤起任务menu；无未结束任务、休眠或拖动时不打开。悬停与Enter/Space保留。

右键入口已通过 macOS 原生真实任务验证，见 `openspec/changes/ds-s2-task-overview/verification-right-click-menu-macos.md`；Windows继续暂缓。

2026-09-14：有任务才悬停、任务状态表现与同桌面面板交互已接线；独立验证见ds-s2-task-overview/verification-task-state-hover-macos.md。空库及原生面板操作PASS，真实运行任务全链路/Windows/完整姿态仍待验证，不Done。

当前卡片操作增量openspec/changes/ds-s2-task-card-actions/；之前ds-s2-task-overview保留功能/验证历史。接管、删除状态由各自Application能力门禁决定，不假装功能已接通。

卡片入口首批及数据保留原生验证PASS：[独立验证](../../../../openspec/changes/ds-s2-task-card-actions/verification-card-actions.md)。接管仍未接通停止/行为记录，按钮存在不等于接管完成；当前所有任务已取消，托盘全部查看历史。

2026-09-14 Agent名称子范围已实施并通过31项核心回归和macOS原生stdio Agent菜单验证，见openspec/changes/ag-s2-local-task-registration/verification-agent-names.md。旧数据备份迁移保留，新测试任务取消后保留；完整Story状态不变，Windows暂缓。

2026-09-15 Agent 当前步骤详情已实施并通过 ego-browser 与 macOS 正式桌面验证，见 `openspec/changes/ds-s2-task-overview/verification-agent-step-ui-macos.md`。详情显示真实标签、step_id 与接受序号；无编辑入口，不把声明冒充执行。
