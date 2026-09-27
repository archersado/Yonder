# CM-S1 产品需求

## 问题与目标

使用 program/args/cwd/env，Shell 必须显式请求，超时可停止进程树且限制输出。

## 范围与非目标

本 Story 仅负责“结构化命令执行”。首批实现 macOS 内部 Runtime/Port，不开放 Shell 或 Agent Gateway；Windows 按用户决定延期并明确 unavailable。不安装新依赖。

## 验收条件

- CM-01：目标行为有可复现成功样本，失败不得伪报成功。
- CM-02：使用绝对`program`、字面`args`、规范化`cwd`和有界`env`，不得用命令字符串代替。
- CM-03：Shell必须使用独立显式入口并取得可信用户确认；Agent字段不能充当确认。
- CM-04：超时或取消停止完整进程树，不把只停止父进程报告为成功。
- CM-05：stdout/stderr分别有界，返回截断事实；日志不记录正文或完整输出。
- CM-06：启动失败、非零退出、超时、取消和输出超限可稳定区分，副作用不明不得自动重试。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

Agent Gateway 的可信风险确认引用、任务事件投影及 Windows Job Object 仍待后续增量；不得用 Agent 自报布尔值绕过确认。

## macOS Runtime 增量（2026-09-25）

来源：CM-01/02/04/05/06、Accepted AD-CM-01 和 macOS Spike。

- CM-RUNTIME-01：只接受规范化绝对 program/cwd、字面 args 和显式有界 env；stdin 关闭、环境清空，不经过 Shell。
- CM-RUNTIME-02：stdout/stderr 并行排空且分别保留最多 64 KiB；任一路超限停止整个进程组并返回超限，不把截断结果当成功。
- CM-RUNTIME-03：超时、取消、输出超限均停止父进程和后代；无法确认进程组消失返回 unknown，不自动重试。
- CM-RUNTIME-04：正常零/非零退出、启动失败、超时、取消、超限和 unknown 稳定区分；输出只在内存结果中返回，不写任务库或日志。
- CM-RUNTIME-05：非 macOS 构建只暴露 unavailable，不注册未验证执行实现。

## 本机批准增量（2026-09-27）

来源：CM-03、产品简报“可见且可控”、Accepted AD-CM-01 和 AD-CM-02。以下为架构定案后的设计细化，不把它表述为新增原始产品需求。

- CM-APPROVAL-01：所有 Agent 结构化命令均须由本机用户在 Task Space 明确批准；Agent、CLI/MCP、Jev 或请求字段不能自报批准。
- CM-APPROVAL-02：批准精确绑定任务、归属 Agent、命令摘要、当前 sequence 和一次性 `command_id`；替换任何 program、args、cwd、env 或 timeout 均须重新提议。
- CM-APPROVAL-03：提议、预览、批准、拒绝、过期、撤权、断连和重启都有稳定可观察状态；未批准前不得启动任务、创建 attempt 或产生副作用。
- CM-APPROVAL-04：完整命令只在本机内存预览中向用户展示，不进入任务事实、事件、Outbox、日志、桌宠或 Agent 响应；Agent 只获得不透明标识与状态。
- CM-APPROVAL-05：执行只接受批准引用；Application 必须先复用 TM-S7 启动事务，再调用 AD-CM-01 Runtime；unknown 不重试。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：首批权限 command:execute、可见且可控。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：Command、File 与 Document。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
