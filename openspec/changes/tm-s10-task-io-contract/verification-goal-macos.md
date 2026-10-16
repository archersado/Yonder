# Verification Goal：TM-S10 任务输入输出与交付契约（macOS）

状态：PASS（2026-10-16）  
环境：macOS；协议 1.52；SQLite schema 27；独立 worktree `story/tm-s10-task-io-contract`  
Windows：按主人既有决定继续暂缓，不纳入本次 macOS 增量结论。

## 目标

验证新任务具有可冻结、可恢复的输入/预期输出契约；成功完成具有有界 Agent 交付摘要；File/Document 写出由可信执行链自动形成 opaque 产物；Task Space 不读取或展示完整 Agent Payload，并保持核心详情先行、附属查询异步加载。

## 自动化证据

- `cargo test -p yonder-protocol -p yonder-cli`：21/21、21/21 PASS；协议 1.52、严格参数、生成模型和 MCP 请求均通过。
- `cargo test -p yonder-application -p yonder-adapters`：92/92、88/88 PASS；覆盖 schema 27 迁移、契约幂等冲突、重启恢复、旧协议降级、同步终态事务、Runtime 结果投影和不可变产物清单。
- `cargo test -p yonder-desktop`：library 39/39、binary 12/12 PASS；发布契约同步为协议 1.52/schema 27。
- `cargo run -p yonder-protocol --example generate -- --check`：PASS。
- `python3 scripts/test_check_architecture.py`：27/27 PASS；`CARGO=/Users/archersado/.cargo/bin/cargo python3 scripts/check_architecture.py`：PASS。
- `openspec validate tm-s10-task-io-contract --strict`：PASS。
- `node --check apps/desktop/ui/task-space.js` 与 `node --check scripts/verification/desktop/check-task-space.mjs`：PASS。

## macOS UI 独立夹具

使用 ego-browser 打开真实 `apps/desktop/ui/index.html`，运行仓库 `checkTaskSpace` 夹具。结果全部 PASS：

- `task.step.get` 返回后立即展示核心详情；`task.events`、Browser、批准、授权和产物仍可独立加载，400ms 延迟不阻塞标题与契约。
- 显示“任务输入”“预期输出”“Agent 交付摘要”；300 字测试输出只显示前 240 字和省略号，完整内容未进入 DOM 文本。
- 旧任务显示“旧任务未声明”，无产物显示“暂无可信产物”，不从 description 或历史正文补造。
- 产物清单空态、版本变化、分页失败恢复和 21 项分页继续通过。
- 全部渲染使用 `textContent`，测试中的 HTML 文本不生成图片或可执行节点。

## 持久化与运行时结论

- `task_io_contracts` 与 `task_results` 只保存有界摘要；创建契约参与 idempotency CAS。
- 兼容执行路径在同一事务写终态、事件、Outbox 与结果摘要。
- Runtime 先发布有序终态/产物事件；异步 Projector 原子写任务状态、事件、Outbox、结果或新清单版本，持久化不阻塞执行循环。
- File 新建/替换、Document 另存仅在动作已观察成功时生成 `artifact_<attempt_id>`；失败、unknown、读取和删除不生成产物。

## 结论

macOS Verification Goal 达成。用户指出的三项问题均闭环：详情附属查询不再造成整体卡顿；完整 Agent 输出不进入 Task Space；任务分配与完成具有明确 I/O/结果契约，真实文件型输出不再长期显示空交付物。Windows 原生证据继续暂缓，因此 Story 保持 `verifying`，不宣称跨平台 Done。
