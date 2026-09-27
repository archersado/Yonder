# TM-S7 统一执行启动独立 Verification Goal

日期：2026-09-21；2026-09-27 补充 Document/Command 交叉验证。关联 TM-S7 TM7-01～06、Accepted AD-TM-13 与 `openspec/changes/tm-s7-unified-execution-start/`。
Result: PASS

## 结果

PASS（统一启动子范围）。`start_execution` 已成为 CUA/BUA 共享入口：`created` 分支同事务迁移为 `running` 并准备 attempt；`running` 分支继续要求已声明步骤、已 Observe/Stopped 边界、同一准入和 CAS sequence。启动成功后才允许 Adapter 派发。

2026-09-22补证：macOS隔离HOME真实桌面进程通过生产UDS完成Browser Gateway生命周期。`tm-s7-check`任务首帧创建后保持`created`，首条`browser.execute`派发前提交执行事实并返回`running`，真实ego-lite外部引用`ego:103`建立；后续已Observe边界推进、同一引用收尾为`completed`。事件链共10条，其中7条为running相关。[结构化证据](../../../apps/desktop/evidence/tm-s7-native-start-macos-20260922/result.json)由[可重复脚本](../../../apps/desktop/evidence/tm-s7-native-start-macos-20260922/check.py)生成。该补证不覆盖Document、Command或Windows。

2026-09-27补证：DO-S2 `task.document.execute` 与 CM-S1 `task.command.execute` 均复用 `start_execution`，真实 macOS desktop Gateway 目标测试通过。Command 在启动前预检本机批准；等待批准样本保持 `created` 且没有 attempt，批准样本提交启动事实后一次消费并真实执行 `/usr/bin/printf`，响应不含完整命令。详情见 [CM-S1 Verification Goal](../cm-s1-agent-command-approval/verification-goal.md) 与 [DO-S2 Verification Goal](../do-s2-agent-gateway/verification-goal.md)。

工作区离线回归 PASS：`cargo test -p yonder-application -p yonder-adapters` 共 39 项通过，其中新增 SQLite 测试覆盖 `created→running`、资源租约保持、已 Observe 边界后声明下一步骤并准备第二个 attempt。`scripts/check_architecture.py` 与 `git diff --check` 通过；`openspec status --change tm-s7-unified-execution-start` 显示 4/4 planning artifacts complete。

## 完成边界

本 Goal 证明 CUA、BUA、Document 与 Command 在当前 macOS 产品路径共享启动语义，但不替代各能力自身的参数、授权、Driver 或 UI 验证。现有 macOS 证据不能替代 Windows；新增 UI、资源或协议变更后仍需分别取得平台证据。TM-S7 保持 verifying，不 Archive。
