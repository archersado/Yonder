# AD-AG-07 慢脑计划统一经 Agent Gateway 接入

状态：Accepted（接入边界；计划协议与事务字段待 EX-S2 联审）
日期：2026-09-21
关联：AG-S1、EX-S2、AD-EX-01、AD-EX-02、AD-TM-08/13

## 决策问题

Accepted AD-EX-01 允许 Yonder 内部以 Jev 连续选择有界动作。2026-09-21 用户再次明确：慢脑的首次计划与 Observe 后 replan 仍沿用现有 Agent Gateway 链路。AG-S1 已有本地 MCP stdio/CLI→Local Socket→GatewaySession，云端设计为单一出站 WSS→同一 GatewaySession；不能为了快脑另建 Agent 通道，也不能让 Jev 冒充外部 Agent。

## 已接受的接入边界

首次计划与 replan 使用同一个版本化计划提交语义，先通过既有会话认证/hello/能力协商，再进入 Gateway 的 Application 用例。Gateway 校验可信会话 `agent_id` 与任务归属、`request_id`/deadline、能力、计划版本及任务当前序号；请求字段不能自报认证身份。Application 决定计划是否可接纳及是否可交给 EX 执行协调层，Gateway 不解释动作、选择 Driver 或持有计划状态。

快脑需要 replan 时，Application 将最小 Observe/失败分类与任务 `sequence` 同事务写入事件/Outbox。原本的 `task.events(after_sequence)` 与云端 Outbox 补传向归属 Agent 提供交回依据；归属 Agent 随后仍通过同一 Gateway 提交新计划。模型服务端不得直接调用 Agent，也不建立第二个 WSS、UDS 或本地 HTTP/TCP 端点。云端 Connector 未就绪时该能力只可在已认证的本地会话上使用，不伪报云端可用。

现有 Agent 逐步提交 `computer.step`、BUA 步骤等能力继续兼容。快脑连续步骤使用 EX-S2 定义的可信 Application 内部来源，不调用 Gateway 伪造 Agent 请求。`DONE` 只形成待验证事实，最终 `task.complete`/`task.fail` 仍由归属 Agent 经 Gateway 提交。

## 对既有 Epic/Story 的调整边界

- AG-S1 保留唯一计划入口，但既有本地 CLI/MCP、私有 stdio 和云端 Spike 验证不因快脑重做；计划协议必须作为后续增量，与 EX-S2 联审后才能生成实施 Proposal。
- AG-S3 既有归属 Agent 步骤声明不返工；EX-S2 引入 Application 内部快脑步骤来源时，必须先与 AG-S3、TM-S2 联审作者、幂等、CAS 与停止排序。
- TM-S2 与 TM-S7 继续拥有任务状态、attempt、Observe、事件、Outbox 和首次副作用启动事务；快脑不得建立平行状态源。计划接受必须与任务事实同事务，且在 AD-TM-13 和 TM-S7 OpenSpec 定案前不实施。
- CU-S2、BU-S2、DO-S2、FI-S1、CM-S1 的 Driver、Bridge、文件锁、命令围栏和双平台验证不变；EX-S3/S4 只能在其 Port 之上做快脑接线，不得让 Adapter 互相调用或绕过 Observe。
- TM-S5、DS-S2、AG-S4 只接收可观察状态与最小交回原因；展示文案、Skill 说明和 UI 状态仍归各自 Story，不能由 Gateway 临时新增私有 UI 状态。

## 定稿门禁

用户已定案「仍从现有 Gateway 进入」，因此本 ADR 接受单一接入边界。EX-S2 必须先定稿计划片段最小字段、版本/CAS、幂等、交回原因和私密内容边界；随后由 AG-S1 联审唯一 Rust 协议模型、能力协商、旧客户端投影、请求/Outbox 大小上限和双平台错误矩阵。计划接受与事件/Outbox 的事务落点须与 TM-S2/TM-S7 联审，不能在 Gateway 新建任务状态源。AD-EX-02 Jev 技术路线和 Windows/macOS 证据未通过前，不开放产品快脑执行；本 ADR 的 Accepted 不授权未设计的协议或实施。
