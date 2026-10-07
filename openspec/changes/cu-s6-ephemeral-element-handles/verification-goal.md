# Verification Goal：CU-S6 临时 Observation transcript 与元素句柄

状态：PASS（macOS 子范围，2026-10-07）。Windows 对等验证按主人决定暂缓，Story 保持 `verifying`，不 Archive/Done。

## 目标

通过正式 Yonder CLI → Agent Gateway → Application → Sky Worker 链路证明：协议 1.44 可把有界、经过安全过滤的当前 transcript、可操作元素句柄和 `observation_ref` 返回归属慢脑；任务持久事件不保存 transcript 或标签正文。

## 自动化结果

- `node --check crates/adapters/src/sky_cua_worker.mjs`：通过。
- `cargo run -p yonder-protocol --example generate --locked -- --check`：通过。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters --locked -- --test-threads=1`：155 项通过（Protocol 18、Application 59、Adapters 78）。
- `openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。

## macOS 正式产品样本

- 构建并打包 `Yonda.app`，由正式 CLI MCP 入口创建任务 `task_debd427edc6b0ece3802b95f49762a8c`。
- 经 `task_plan_submit` 与 `task_plan_execute` 启动企业微信；返回协议 1.44 Observation，包含 `element_count=260`、30 个有界可操作句柄、非空 `observation_ref` 和非空 transcript。
- transcript 与标签正文不写入本文件。随后读取 `task.events(after_sequence=0)`，序号 1～7 只包含创建、步骤、attempt 与状态事实，不含 Observation transcript、元素标签或截图正文。
- 使用 Observation 序号 5 提交目标核验，任务在序号 9 正常完成；证明临时响应不阻塞既有 Goal/完成链路。

## 结论与边界

macOS 子范围通过 OEH-01～OEH-08，并证明 OEH-09 的正式链路能返回可供慢脑消歧的 transcript 与句柄；OEH-09 的企业切换点击推进仍需后续副作用样本。本 Goal 不证明 Windows 等价行为；Windows 验证继续暂缓，不能由本结果外推。
