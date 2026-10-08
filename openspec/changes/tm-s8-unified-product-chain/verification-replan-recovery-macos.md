# TM-S8 首动作前交回后 replan 恢复 macOS 验证

日期：2026-10-08  
平台：macOS  
基线：`dev@92d1f22`  
结论：通过（仅关闭 replan 恢复边界）

## 验证目标

证明同一任务在首个 Driver 动作前被 Jev 交回后，即使没有历史 attempt、也没有既有 Admission，归属慢脑仍可经 Gateway 提交新片段，并由事件驱动 ExecutionRuntime 在首个副作用前安全恢复执行。不得把任务退回 `created`、创建第二任务，或绕过桌面单租约。

## 回归场景

`plan_fragment_continuously_executes_verified_slots_in_one_call` 新增以下精确边界：

1. 创建任务并提交第一版计划片段；
2. 在任何动作 attempt 产生前交回慢脑，断言任务为 `running` 且 attempt 为空；
3. 同一归属 Agent 为同一任务提交第二版计划片段；
4. 使用全新的 ExecutionRuntime 与空 Admission 执行该片段；
5. 断言 Runtime 先恢复投影并取得 Admission，随后连续派发 `launch_app`、`bring_to_front` 两个槽位；
6. 断言结果进入 `awaiting-goal-verification`，没有再次触发旧同步入口的 `StopRequired/-32012`。

## 验证命令与结果

- `cargo test -p yonder-adapters plan_fragment_continuously_executes_verified_slots_in_one_call -- --nocapture`：1/1 通过。
- `cargo test --offline --locked -p yonder-application -p yonder-adapters`：Application 73/73、Adapters 83/83 通过，Doc tests 通过。
- `python3 scripts/check_architecture.py`：通过。
- `openspec validate tm-s8-unified-product-chain --strict`：通过。

## 边界

本记录证明实现层恢复边界与回归保护，不替代真实 Codex MCP、正式 Yonder GUI、Jev 远端选择及 CUA Driver 的产品级重跑。原失败记录保留为历史证据；CUA、BUA、Office、Command 完整闭环和快慢脑正式配对样本仍须分别完成，Windows 继续暂缓。
