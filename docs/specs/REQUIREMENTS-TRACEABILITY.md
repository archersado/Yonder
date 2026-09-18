# 产品与架构来源映射

本文是拆解索引，不替代或改写原文。用户确认以 `_bmad-output/planning-artifacts` 下产品简报和架构材料为基线。

## 基线文件

- [B：产品简报](../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)：产品定义、MVP 主干链路、权限、桌宠、成功标准与非目标。
- [A：产品补充材料](../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)：接入方式、Task Space 容器和 CUA/BUA 边界。
- [S：架构主干](../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)：职责、依赖、协议、持久化、安全、平台和发布约束。
- [D：研发模式](../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/DEVELOPMENT-AND-CHANGE-MODE.md)：Story 设计先于 Proposal 与实施。

各 ADR 的 Accepted/Proposed 与平台范围必须逐项遵守，后来的拟议文档不自动覆盖基线。记忆日志仅用于追溯，不替代上述材料。

## 模块覆盖与仍须拆解的范围

| 来源章节 | 产品/架构要求 | 归属 | 当前差距 |
|---|---|---|---|
| B 产品定义、桌宠；S Recording 与桌宠 | 可见控制、标准动画资源包导入 | DS | DS-S1 只有桌宠/宿主基础，资源包导入尚未独立拆分 |
| B MVP 主干 1–2；S 非功能与发布 | 安装、账号连接、明确权限、双平台分发 | DS / AG / EN | 账号、连接管理、撤销和签名发布尚未细拆 |
| B Task Space 与权限模型；A ego-lite 接入参考；S Agent Gateway | 开放 Agent 接入、权限分离、连接发现与控制 | AG | AG-S1～S3已覆盖本地接入、创建和步骤增量；云端接入仍缺分解 |
| A Agent Skill + 本地主机 CLI + 应用内 Runtime；用户2026-09-17变更 | 统一Yonder Skill覆盖BUA、CUA、文档和命令；BUA嵌入ego-browser规则 | AG-S4 | 规格已拆解；等待四类Gateway能力完成后再生成OpenSpec和Skill包 |
| B Task Space 与权限模型；A CUA 与 BUA 的 Task Space；S 任务、状态与恢复 | 发起者、权限、生命周期、资源、时间线、产物、审计 | TM / DS | TM-S1 目前字段与范围不完整，不能只保留状态和最新详情 |
| B MVP 主干 7–8；S 任务、状态与恢复 | 暂停、取消、接管、明确恢复 | TM-S3 / TM-S4 / CU-S2 | 停止确认、控制语义与 UI 仍待设计 |
| B 两条执行路径；A 执行原则；S CUA 与 BUA | 模型无关 Driver、Observe、单前台租约 | CU / BU | Spike 结论不能替代产品 Driver/Bridge Story |
| 用户2026-09-18后台原生动作变更；B任务可见可控；S任务恢复 | 原生API/App Intent/SDK后台执行，显式切换CUA，统一Task Space时间线 | CU-S3 / TM-S5 | 已完成Story与Proposed AD-CU-06；先做macOS限时Spike，Windows暂缓，未接产品Gateway |
| B 首批权限；S Command、File 与 Document | 结构化命令、路径安全、文档默认另存与锁 | CM / FI / DO | 产品协议、确认与失败场景待细拆 |
| B 产品定义、MVP 主干 3–5；S 个人上下文 | 显式录制、授权查询、同步外部上下文服务 | RC / CX / AG | CX-S1 是采集 Spike，历史检索/客户端同步/删除传播未细拆 |
| B MVP 主干 9；A Task Space | 时间线、产物、结果检查与审计历史 | TM-S5 / DS-S2 / RC | 已补 TM-S5 三份设计与 9 条验收；事件/产物/确认契约仍待联审，不能用最后一条摘要代替 |
| S 个人上下文、非功能与发布 | 配额、保留、加密、备份、升级 | ST / EN / CX | ST-S2 密钥暂停，其余配额/迁移/发布未细拆 |
| 用户后续确认（已记录 AD-OCT-03） | 所有任务统一展示、资源约束并行 | DS-S2 / TM-S2 | 与早期“先主干后并发”区分，遵守最新明确要求 |
| 用户桌宠交互与暂缓指令 | 三分钟/忙碌不隐藏/边缘双眼/环绕菜单 | DS-S1 / DS-S3 | 原产品简报没有这些细节；环绕菜单只记规格，暂停实现 |
| 用户2026-09-17插件迁移要求；参考`learn/avatar-orb-pet` | Windows/macOS单轮语音、会议双路转写、屏幕圈选后提问 | VI-S1 / VI-S2 / CX-S2 | 属后续需求；CX-S2已建立Proposed AD-CX-01与区域捕获Spike，macOS子范围待验证，Windows暂缓；产品提交仍受AG-S5通道门禁 |
| 用户2026-09-17 Record & Replay借鉴要求；参考`learn/avatar-orb-pet/lib/rpa.js` | 录制后步骤审阅、版本化Trajectory、确认回放、倒计时与随时停止 | RC-S1 / RC-S2 / CU / TM | RC-S2已拆解；来源识别、正文持久化、Trajectory协议和双平台回放仍待ADR与验证 |

## 冲突与优先处理

1. B 说默认不采集、无持续后台监控，S 描述事件驱动窗口/浏览采集。CX/RC 设计必须明确启用条件与授权生命周期，未解决前不能默认后台启动采集。
2. B/A 描述隔离浏览器任务可并行，当前 AGENTS/架构实施规则限制 BUA MVP 单并发。保留能力方向，当前实现按单并发，不偷换为已实现多浏览器并行。
3. 原产品目标要求 Windows/macOS，部分已接受 ADR 只覆盖 Windows。延期不删除产品平台要求，不把单平台证据称为整体完成。
4. Task Space 历史与审计是原需求。AD-TM-01 的仅最新详情建议不能自动覆盖；先分清最新快照和历史事件/产物职责，再定持久化设计。

当前 13 个 Epic、31 个 Story 仍不是完整产品分解。TM-S1 已补到 16 条验收，时间线/产物/审计归 TM-S5；其他模块缺口继续逐步拆解。本表不授权新实施。

TM-S6 承接 TM-S1 AC11 的全量数据库查询子范围，按 AD-TM-02 独立实施；未关闭完整忙碌/隐藏门禁。
