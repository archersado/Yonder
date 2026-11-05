# CU-S4 产品需求

## 问题与目标

验证 `vercel-labs/agent-desktop` 能否作为 Yonder 的低层 CUA Driver：在 Windows/macOS 上提供有界 Observe 与原子动作，同时把任务控制、人类输入抢占、取消、状态、审计和恢复完整保留在 Yonder Application。

## 原始需求、后续变更与设计建议

- 原始需求：产品简报“两条执行路径”和补充材料“执行原则、验证方案”要求模型无关 CUA、动作后 Observe、故障不误报、用户可暂停/取消/接管。
- 后续用户变更：2026-11-05 明确把 `agent-desktop` 当作 Driver 使用，而不是完整 Agent、任务控制器或第二 Gateway。
- 已接受架构约束：AD-CU-02/04 与 AD-TM-08 规定 Application 拥有派发准入和停止事实，用户输入冻结新动作，超时/断连/崩溃/交付不明为 `unknown` 且不自动重试。
- 待审设计建议：固定 `v1.1.0` commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`；使用隔离 Worker 内最小 Rust bridge + inherited stdio JSONL；五个工作日和统一样本阈值来自研究建议，不称为原始产品要求。

## 范围

- Windows/macOS 使用相同语义验证 snapshot、qualified ref、目标重识别、原子动作、动作后 Observe、跨进程 interaction lease 和 delivery disposition；固定任务集含 Notepad/TextEdit、Electron fixture、Windows WPS/文件资源管理器与 macOS Office/Finder。
- 验证外部监督能否实现用户真实输入抢占、派发冻结、步骤间取消、在途 Worker 终止、资源清理与 `unknown` 映射。
- 验证固定源码自建、干净机运行、SBOM/许可清单及 Yonder 自有 Windows/macOS 签名责任链。
- ADR 结论只能是“双平台共享并 Accepted”或“整体 Rejected”；不得建立单平台长期双栈。

## 非目标

- 不替换 trycua，不改产品依赖或 CUA Adapter。
- 不接 Agent Gateway、任务协议、SQLite、Task Space、桌宠或真实用户任务。
- 不实现模型循环、语义规划、上游 MCP、CDP 或本地 HTTP/TCP。
- 不承诺原生动作可回滚，也不把 Worker 退出解释为副作用取消成功。

## 验收条件

- `ADS-01`：候选仅在受监管独立 Worker 中运行；Desktop 主进程不加载候选引擎或最小 Rust bridge，通信不开放 HTTP/TCP。
- `ADS-02`：Application/探针只通过既有 ComputerUse Port 语义调用候选，Domain/Gateway/持久化不引入上游命令或状态模型。
- `ADS-03`：正常动作后必须新鲜 Observe；只有 Observe 验证后才可报告已知成功。
- `ADS-04`：真实用户输入后先冻结下一次派发；输入分类仅允许 `driver_current_attempt | user | unknown | observer_lost`，固定样本须零漏分、零误分，observer 异常 fail closed，否则淘汰。
- `ADS-05`：未派发与步骤间取消可确定完成；在途终止只要可能交付即返回 `unknown`，不得自动重试。
- `ADS-06`：同名窗口、窗口重建/句柄复用、前后台变化中不得误投递；stale、ambiguous 或身份不明必须拒绝。
- `ADS-07`：Worker 必须被平台 containment；终止后 interaction lease、按键/鼠标状态、子进程与临时资源全部以只读探针证明释放。不能证明则进入全局桌面 quarantine 并淘汰，禁止合成补偿输入。
- `ADS-08`：Windows/macOS 使用同一结构化样本与判定矩阵，分别提交原生证据。
- `ADS-09`：固定源码、patch set、依赖锁、toolchain、features、targets、CRT/链接策略可重现自建；干净 Windows 无未声明运行库，并产出 SBOM/许可清单、provenance、Windows Authenticode 与 macOS Developer ID/hardened runtime/notarization/stapling/Gatekeeper 分层结论。
- `ADS-10`：所有 hard gate 均通过后才能提出产品 Apply；任一硬淘汰条件命中则候选 Rejected，不能用降低范围掩盖失败。

## 需求来源与验收映射

| 来源 | 章节/变更 | 映射 |
|---|---|---|
| [产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md) | 两条执行路径、MVP 主干链路 | ADS-02、03、08 |
| [补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md) | 执行原则、验证方案 | ADS-03～08 |
| 用户 2026-11-05 变更 | “相当于把它当 driver 使用” | ADS-01、02、10 |
| [架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md) | CUA 与 BUA、任务状态与恢复 | ADS-01～08、10 |
| [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md) | 候选边界、抢占取消、淘汰门槛 | ADS-01～10 |
| [独立研究](../../../../_bmad-output/research-vercel-labs-agent-desktop-yonder-cua/research-vercel-labs-agent-desktop-yonder-cua.md) | 硬门槛、外部控制、建议 | ADS-01～10 |
