# Verification Goal：TM-S3 显式 CUA 接管与顶部控制条

状态：PASS（macOS 产品范围）；日期：2026-09-28。Windows 按主人既有决定暂缓，不外推为双平台通过；TM-S3 完整 Story 仍保留 Windows、Recording 与交回剩余范围。

## 自动验证

- `node apps/desktop/check-cua-control.mjs`：首帧主动回放、慢脑规划、快脑决策、执行步骤、唯一显式接管、等待态与失败态全部通过。
- `cargo test -p yonder-application -p yonder-adapters -p yonder-desktop --offline --locked`：Application 45 项、Adapters 70 项、Desktop lib 17 项、Desktop main 11 项通过，含跨步骤 Hub、无参数决策投影、直接运行中取消与一次性接管消费。
- `cargo build -p yonder-desktop --locked` 与 `git diff --check`：通过。
- `python3 -m unittest test_check_architecture.py`（`scripts/`）：19 项通过。仓库全量 `check_architecture.py` 仍被既有 EX-S2 Story/Proposal 双向关联缺口阻断，与本变更无关，未修改其范围。

## macOS 独立原生验证

既有 `tm-s3-explicit-cua-takeover-macos-20260928` 证据继续有效。本轮新增两组正式 bundle/UDS 验证：

- `check-cua-control-idle-macos.py`：真实 `computer.step` 响应返回后停止发送 Gateway 帧，控制条仍通过当前投影回放显示；AXPress 后任务为 `paused`、控制为 `stopped`，证明步骤间接管不依赖下一请求。
- `check-cua-control-macos.py`：慢脑经 Gateway 提交 8 槽位计划，首槽位含两个候选并真实调用面板 Jev；控制条显示慢脑步骤、Jev 实际 HandBack 决策与当前执行步骤。观察结束后正式 `task.cancel` 清理，状态为 `cancelled`。
- 两组窗口均为 560×174、当前工作区顶部 16pt 横向居中；AX 同时找到“慢脑规划”“快脑决策”“正在执行”和真实步骤文本。
- 结构化投影不包含候选参数、消息正文、完整模型响应或思维链。

证据见 [`步骤间接管`](../../../apps/desktop/evidence/tm-s3-fast-slow-idle-macos-20260928/README.md)与[`快慢脑投影`](../../../apps/desktop/evidence/tm-s3-fast-slow-plan-macos-20260928/README.md)。

## 2026-09-29 步骤结果真实性增量

- `cargo test -p yonder-application -p yonder-adapters -p yonder-desktop`：Application 45、Adapters 70、Desktop 19 项通过。Adapter 合约覆盖 `confirmed`、`refused`、`partial`、`unverifiable`、`suspected_noop`、缺失 effect 与字段矛盾；后三类及无效响应不得写成成功。
- `node apps/desktop/check-cua-control.mjs`：绿色完成、当前执行、待核实及等待状态的可访问文案通过；Hub 回归证明不会由下一步开始反推前序成功。
- `check-cua-dispatch-macos.py`：trycua 0.25.0 对隔离原生文本框返回 confirmed 时，原生目标值同时匹配，结果才持久化为成功。
- `check-cua-step-status-macos.py` 经正式 `Yonda.app`、Local Socket 与真实 trycua 运行：confirmed/CAS 推进样本显示可访问绿色 ✓；另一轮 Worker 不可确认时记录 unknown、片段 HandBack 并显示可访问黄色待核实 !，没有伪造成功。两轮验证任务均经正式 `task.cancel` 清理。

证据见 [`成功边界`](../../../apps/desktop/evidence/tm-s3-cua-step-status-macos-20260929/README.md) 与 [`待核实边界`](../../../apps/desktop/evidence/tm-s3-cua-step-unverified-macos-20260929/README.md)。Windows 按既有决定暂缓。
