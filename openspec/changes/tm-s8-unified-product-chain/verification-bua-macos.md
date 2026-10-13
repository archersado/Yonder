# TM-S8 BUA 正式产品闭环 macOS 独立验证

状态：PASS（BUA macOS 子目标）；日期：2026-10-13。Windows 继续暂缓；CUA、Office、Command 与 TM-S8 总体验证不因本结果提前通过。

## 验证边界

- 任务由当前 `codex-cli` 会话通过安装包发布的 Yonder MCP 创建；Yonder 负责任务状态、执行尝试、Observe、ego-lite Task Space 引用、交回/恢复和终态。
- 页面动作只发生在 Yonder 返回的唯一 ego-lite Task Space 中；没有另建浏览器、直连 Gateway Socket、读取 SQLite 或调用私有 Browser Adapter。
- 样本使用无登录、无写入、无账号副作用的公开文档示例页。证据不保存页面正文、完整 URL、截图或浏览器 Payload。

## 正向任务

任务 `task_25cd349a09dd03df2e4f8dde06833279` 在 `created@1` 后声明“打开公开测试页并验证内容”，由 `browser.execute(create)` 创建唯一引用 `ego:34` 并进入 `running@3`。同一空间的唯一 Page 成功取得预期标题和公开说明事实，随后经 Yonder `browser.execute(observe)` 写回执行边界，并由 `browser.execute(finish)` 结束空间和任务。最终通过 `task.get` 与 `task.events` 复核为 `completed@14`。

首次等待预期 heading 超时后没有新建空间、没有盲目重放动作；只读取同一 `ego:34` 的当前 URL、标题与语义快照，确认页面已经加载而站点结构不再包含该 heading，再继续完成。这一负分支证明页面动作回执不会被直接当成目标成功。

## 交回与同空间恢复

任务 `task_35763cdc782acc8b19cb5de4ca5eb876` 创建唯一引用 `ego:35`，在同一 Page 取得目标事实并 Observe 后，经 Yonder 执行：

1. `hand-off`：引用 ownership 变为 `agentDelegatedToUser@12`，任务保持 `running`；
2. `take-over`：ownership 恢复为 `agent@15`；
3. 恢复后仍为 `ego:35`、唯一托管 Page，标题事实保持；
4. `finish` 后空间标记 `finished=true`，任务为 `completed@20`。

该样本没有创建第二 Task Space，也没有在交回期间绕过 Yonder 操作页面。

## 结论

TM8-01、TM8-03、TM8-06、TM8-07 在 BUA macOS 子范围取得正式产品证据：真实 Codex MCP、Yonder GUI 任务、统一 Gateway、ego-lite Task Space、Observe、交回/恢复和终态可形成闭环。BUA 当前没有 Jev 候选决策，因此本样本不用于证明 TM8-08、TM8-09 或快慢脑性能优势。

脱敏结构化证据：`apps/desktop/evidence/tm-s8-bua-product-chain-20261013/result.json`。
