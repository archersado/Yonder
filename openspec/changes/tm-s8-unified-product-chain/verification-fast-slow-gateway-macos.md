# TM-S8 快慢脑正式 Gateway 链路 macOS 验证

日期：2026-09-28  
平台：macOS  
基线：`dev@3d9a907`  
结论：**FAIL（已定位实现阻断，不得作为通过证据）**

## 目标链路

本轮只接受以下产品路径：

`Codex 慢脑 → yonder MCP stdio → 私有 UDS → Agent Gateway → 不可变计划片段 → Jev 有界选择 → Yonder CUA Driver → Observe → 事件/Outbox → 同一 Gateway 交回或终态`。

正式样本不使用 Codex Computer Use 执行或准备 CUA。安全目标应用也由 Yonder 自己经 `computer_step/launch_app` 启动；早先使用外部 Computer Use 激活夹具的尝试已作废，不计入下述证据。

## 环境与前置检查

- Jev 面板配置：`enabled=true`、`service_mode=remote`、端点为 `https://api.typesafe.ai`，CUA/BUA/Document/Command 四类能力均启用。
- Keychain Adapter 真实远端探测成功，未读取或输出 API Key；Jev 服务可访问。
- Yonder CUA 目标准备任务 `task_d7e835b54a84ca49cbb9d2e91135fc72` 通过正式 MCP/Gateway 调用 `launch_app(com.apple.TextEdit)`，动作 `action_succeeded=true`、完成 Observe并到达 `completed`；交回后的第二次目标恢复任务 `task_204ca865f209e70500aafae1434cbe29` 同样通过。

## 正式多候选片段

Codex 身份 `codex-cli` 创建任务 `task_cd6eb093c5872cf5f3b23c27217c5a4d`，经 MCP/Gateway 提交计划 `codex-jev-cua-plan@1`：

1. 第一槽位含两个受限候选：TextEdit 安全输入候选与前置条件为 false 的无关候选；Application 另加入唯一 `handback`。
2. 第二槽位是确定性的安全 `press_key(tab)`，用于验证同一次 Gateway 调用内的连续执行。
3. 提交返回 `accepted@2`，证明慢脑计划从既有 Gateway 进入并完成不可变持久化。
4. `task.plan.execute` 真实调用 Jev，约 1174 ms 后返回 `handback@3`；没有派发 CUA，`observed_attempts=0`，符合不确定时不得误操作的安全约束。
5. Gateway 随后读回任务为 `running@3`，`next_intent=需要慢脑重新 Observe 或规划`；事件与 Outbox 均保留创建、提交、交回事实。

## 交回后 replan 阻断

Codex 从同一 Gateway 读取交回后的任务，再提交确定性的 `codex-replan-after-handback@1`。片段被接受为序号 `4` 并持久化，`current_slot=0`；随后执行返回：

```json
{
  "code": -32012,
  "message": "当前步骤结果待核实或尚未到达安全边界，无法取消"
}
```

该错误文案来自通用 `StopRequired` 映射，实际阻断已由代码路径和运行事实共同定位：

1. `hand_back_plan_fragment` 按 AD-TM-21 把 `created` 原子变为 `running`，这是正确的状态同步。
2. Jev 在首个 Driver 动作前交回，因此任务没有执行 attempt，也没有取得桌面 Admission。
3. replan 的首个动作进入 `start_execution`；该函数对 `running` 任务只允许 `admission.holds(task_id)==true` 的连续动作。
4. 当前任务没有 Admission，故固定返回 `StopRequired`；现有存储恢复分支同时要求一个已停止的历史 attempt，首动作前交回也不满足。

因此链路停在“交回后重新执行”，无法到达 Jev 正向选择、连续 CUA Observe 与归属 Agent 提交终态，TM8-08/TM8-09/TM8-10 尚未通过。

## 修复验收要求

- 保持交回后任务为 `running`，不得退回 `created` 或创建第二状态机。
- 仅对“归属 Agent、已接受的新片段、无 pending control、无未决 attempt、无既有 Admission”的首动作前交回状态开放一次安全 Admission 恢复。
- 恢复仍须经过任务 CAS、桌面单租约、正式 CUA Driver 和每步 Observe；不得后台重试。
- 重跑本文件的多候选样本，必须同时证明 Jev 真实选择受支持候选、至少两个槽位连续 Observe、异常交回可再次 replan，以及最终由 `codex-cli` 经 Gateway 提交任务终态。
- 记录与纯 Codex 逐步决策的分段耗时；本次 FAIL 不作性能优越性结论。

复现入口：[scripts/verify_tm_s8_fast_slow_macos.py](../../../scripts/verify_tm_s8_fast_slow_macos.py)。
