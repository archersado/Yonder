# Verification Goal：TM-S3 显式 CUA 接管与顶部控制条

状态：PASS（macOS 产品范围）；日期：2026-09-28。Windows 按主人既有决定暂缓，不外推为双平台通过；TM-S3 完整 Story 仍保留 Windows、Recording 与交回剩余范围。

## 自动验证

- `node apps/desktop/check-cua-control.mjs`：控制条开始、步骤摘要、唯一显式接管、等待态与失败态全部通过。
- `cargo test -p yonder-application gateway::tests::execution_notification_only_accepts_decoded_execution_requests --locked`：只有真实 CUA 请求产生控制提示。
- `cargo test -p yonder-adapters plan_fragment_continuously_executes_verified_slots_in_one_call --locked`：显式信号在已观察动作后阻止下一计划槽位。
- `cargo test -p yonder-desktop --lib --locked`：12 项通过，含活动任务绑定、错误任务拒绝和一次性消费。
- `cargo build -p yonder-desktop --locked` 与 `git diff --check`：通过。
- `python3 -m unittest test_check_architecture.py`（`scripts/`）：19 项通过。仓库全量 `check_architecture.py` 仍被既有 EX-S2 Story/Proposal 双向关联缺口阻断，与本变更无关，未修改其范围。

## macOS 独立原生验证

`python3 apps/desktop/check-cua-control-macos.py apps/desktop/evidence/tm-s3-explicit-cua-takeover-macos-20260928` 通过：

- 正式打包 `Yonda.app`、真实 UDS、协议 1.31 和真实 trycua 连续计划链路；
- 控制条 460×68，当前工作区顶部 16pt 横向居中，与圈选工具条定位一致；
- 文案与“接管电脑”按钮可访问，真实 AXPress 成功；
- 计划返回 `takeover-requested`，未执行剩余全部槽位；
- 任务进入 `paused`、控制进入 `stopped`，新增 `user-input` unknown 数为 0；
- 用户现场确认顶部交互位置与圈选样式符合预期。

证据见 [`README.md`](../../../apps/desktop/evidence/tm-s3-explicit-cua-takeover-macos-20260928/README.md)。
