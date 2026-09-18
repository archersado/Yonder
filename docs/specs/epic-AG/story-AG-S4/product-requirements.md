# AG-S4 产品需求

## 问题与目标

Agent 需要一个可安装的 Yonder Skill，统一说明如何通过 Yonder 完成浏览器、桌面、文档和命令行任务。使用者只面对一套任务生命周期，不需要了解 Yonder 内部 Gateway、ego-lite Task Space 或 CUA Worker 的组合细节。

BUA 部分复用并嵌入 ego-browser Skill 的有效操作规范；ego-lite 仍是浏览器执行 Runtime，Yonder 负责可信任务、外部 Task Space 引用、资源准入、Observe、控制、事件和完成状态。Skill 不复制 ego-lite，也不能让 Agent 创建一个未关联 Yonder 任务的独立浏览空间。

## 范围与非目标

- Yonder 连接检查、能力发现、任务创建、步骤、事件、取消、暂停、接管和完成。
- BUA：在 Yonder 分配并登记的 ego-lite Task Space 中执行，继承 ego-browser 的单空间、页面复用、用户交接、可观察动作和结束规则。
- CUA：只通过 Yonder 暴露的模型无关 Driver 工具执行，每步由 Yonder 完成步骤声明、动作后 Observe 和边界推进。
- Document：只调用届时由 Yonder Gateway 发布的 Document 能力，遵守另存、锁、`expected_hash`、结构校验和原子替换。
- Command：只调用届时由 Yonder Gateway 发布的结构化 `program + args + cwd + env` 能力；Shell、提权、安装、删除、支付和发送继续走明确确认。
- Mascot generation：依据 DS-S4/AD-DS-04，使用 Hatch Pet 式流程将用户明确授权的形象参考生成 Yonder 九个标准状态动画，并输出可导入 manifest 包。

### 非目标

- Skill 不包含 Planner、模型、云端 Agent 服务、SQLite 访问、Socket 私有协议或第二份任务状态。
- Skill 不携带 CUA Worker 代码，不重定义 trycua 动作及参数 Schema。
- Skill 不实现浏览器、文档或命令 Runtime，不用脚本、系统命令或其他浏览器绕过缺失的 Yonder 能力。
- 本 Story 不提前规定尚未定稿的 Document/Command 工具名称和协议字段。
- Skill 不在未经用户确认时上传形象参考，不在 Yonder 中保存模型密钥，不把 Hatch Pet 图集直接当作可运行包。

## 验收条件

- SKILL-01：一次用户目标只创建一个 Yonder 任务；名称由 Agent 根据目标生成，幂等重试不产生重复任务。
- SKILL-02：Skill 启动时读取 Yonder 实际暴露的能力；能力缺失时明确报告，不直接调用替代执行栈冒充成功。
- SKILL-03：BUA 只能使用 Yonder 返回并持久化的 ego-lite Task Space 引用；不得另建未关联空间。每次有效动作后 Observe，用户接管时停止 Agent 操作，完成时由 Yonder关闭任务引用。
- SKILL-04：嵌入的 ego-browser 规则有明确上游版本或提交来源；保留单 Task Space、恢复同一空间、Page 复用、`handOff/takeOver` 和完成清理语义。升级必须审阅差异，不能静默覆盖 Yonder 围栏。
- SKILL-05：CUA 使用 Yonder 的组合步骤入口，动作名和参数只来自运行时 SDK 能力；Skill 不维护第二份动作枚举。用户输入、unknown 或 Observe 失败不得自动重试副作用。
- SKILL-06：Document 与 Command 仅在其正式 Gateway Story 和独立 Verification Goal 通过后启用；文件锁、另存、哈希、结构化命令和确认边界不可由 Skill 简化。
- SKILL-07：取消、暂停、接管、交回和完成使用 Yonder 当前任务序号及结构化结果；失败保留可恢复事实，不用自然语言猜测任务状态。
- SKILL-08：Skill 不记录正文、截图、输入、完整命令输出或完整 Agent Payload；输出仅包含完成任务所需的有界结果与引用。
- SKILL-09：安装包、仓库发布物或 Skill 市场中的包内容一致，入口可被 Agent 自动发现；卸载 Skill 不影响 Yonder 桌面数据与 Runtime。
- SKILL-10：使用真实 Yonder Runtime 分别验证 BUA、CUA、Document、Command 成功与拒绝路径；涉及原生 UI/Driver 的平台证据遵守各能力 Story 的完成门禁。
- SKILL-11：形象生成先锁定用户授权的角色身份，再逐状态生成、透明度/循环/连续性质检；只将通过 DS-S4 manifest 校验的包交回 Yonder，本次外发取消或失败不替换当前形象。

## 需求来源与验收映射

- [产品简报补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)「ego-lite 接入参考」与“Agent Skill + 本地主机 CLI + 应用内 Runtime”→SKILL-01～04、09。
- [产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)「MVP 主干链路」「Task Space 与权限模型」「首批权限」→SKILL-01、05～07、10。
- [架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「Agent Gateway」「CUA/BUA」「Command/File/Document」「任务、状态与恢复」→SKILL-02～08、10。
- 2026-09-17 用户变更：为 Yonder 生成统一 Skill，包含使用 ego-lite 的 BUA、CUA、文档和命令行；BUA 部分嵌入 ego-browser Skill。用户随后要求当前只写规格，等待四类能力全部实现后再补 Skill→本 Story 的范围及实施门禁。
