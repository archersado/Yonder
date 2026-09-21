# TM-S7 产品需求

## 问题与目标

用户创建任务后，只要任一获准执行能力开始派发副作用，任务就必须进入真实的 `running` 状态。CUA、BUA、Document 与结构化 Command 不得各自定义“执行中”，也不得在仍显示 `created` 时操作设备、浏览器、文件或进程。

## 范围与非目标

本 Story 规定首次执行的统一生命周期：声明步骤、原子启动、资源准入、派发、Observe/结果记录及停止或终结。覆盖 CUA、BUA、Document、Command 四类能力的共同约束；各能力的参数、文件语义、命令协议与具体 UI 仍归 CU、BU、DO、CM。

不新增任务状态、不把执行中的子状态持久化为第二事实源、不自动恢复或重试 unknown、不绕过 Office/WPS 锁或用户确认。

## 验收条件

| 编号 | 场景 | 期望 |
|---|---|---|
| TM7-01 | `created` 任务首次执行任一能力 | 在派发前同事务写入 `running`、步骤、尝试与 Outbox；提交失败则不得派发。 |
| TM7-02 | 已为 `running` 的任务执行下一步 | 复用同一任务与准入事实，只能在已 Observe 的步骤边界推进。 |
| TM7-03 | CUA、BUA、Document、Command | 共享同一启动用例和状态语义；能力仅声明自身资源与副作用契约。 |
| TM7-04 | 启动/派发/Observe 任一阶段失败或断连 | 返回可区分错误；副作用未知为 `unknown`，不自动重试，不伪报完成。 |
| TM7-05 | 用户输入、暂停、取消或接管 | 只在既有控制契约允许的边界停止；停止确认前仍是执行占用，UI 不误报空闲。 |
| TM7-06 | 桌宠与 Task Space | `running` 后展示执行中；`created` 仅展示已创建未执行；无能力或门禁未满足明确提示，不用执行动画掩盖。 |

## 需求来源与验收映射

- 原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)「MVP 主干链路」第 5–9 步、「Task Space 与权限模型」「两条执行路径」；映射 TM7-01 至 TM7-06。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「任务、状态与恢复」「CUA 与 BUA」「Command、File 与 Document」；映射 TM7-01、TM7-03 至 TM7-05。
- 后续用户变更：本次明确“创建后只要开始 CUA/BUA/文档/本地命令执行即为执行中”；映射 TM7-01、TM7-03、TM7-06。
- 已定案设计建议：共享 `start_execution` 用例名称、能力资源枚举与协议字段，由 Accepted AD-TM-13 定案，不冒充原始产品需求。
