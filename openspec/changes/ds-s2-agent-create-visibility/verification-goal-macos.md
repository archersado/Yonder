# Verification Goal：DS-S2 Agent 新建任务即时透出（macOS）

状态：PASS（2026-09-28）  
验证者：独立产品链路验证  
环境：macOS，正式调试打包 `target/debug/Yonda.app`，协议 1.32，SQLite schema 21  
Windows：依主人决定暂缓，不纳入本次 macOS 结论。

## 目标

验证 Agent 通过打包 CLI/MCP、Local Socket 与统一 Gateway 创建任务后，Yonder 自动展示任务空间“全部”视图，第一页按可信创建时间倒序，最新任务位于首项；不由前端伪造卡片。

## 自动化证据

- `cargo test -p yonder-adapters --lib --locked`：70/70 PASS；含创建时间倒序、同时间 ID 并列、三页排他游标、历史未知时间、协议 1.31 拒绝及 1.32 Gateway 成功。
- `cargo test -p yonder-application -p yonder-protocol --locked`：44/44 与 13/13 PASS。
- `cargo test -p yonder-desktop --locked`：library 15/15、binary 11/11 PASS；包含 CUA 控制条不压制新任务、语音/圈选保护与发布契约 1.32/schema 21。
- `cargo run -p yonder-protocol --example generate --locked -- --check`：PASS。
- `node --check apps/desktop/ui/task-space.js` 与 `node --check apps/desktop/check-task-space.mjs`：PASS；前端夹具断言每次列表请求均携带 `newest_first=true`。

## 正式链路证据

1. 重新构建、adhoc 签名并启动正式 `Yonda.app`。
2. 使用包内 `yonder mcp`、已启用 Agent `codex-cli` 调用 `task_create`，名称为“现场试用 4：最新任务在第一页”；返回真实 `snapshot`，状态 `created`、序号 `1`、来源 `local-agent`。
3. 原生 CUA 读取 `Yonda · Task Space` AX 树：
   - “全部”切换为 on，“进行中”为 off；
   - 列表首张卡片为“现场试用 4：最新任务在第一页 · Agent codex-cli · 已创建”；
   - 页签为“第 1 页 · 20 项”，且“下一页”可用，证明不是清空历史或绕过分页；
   - “取消任务”可用、“接管”按真实 created 状态禁用。
4. 再经包内 MCP 调用 `task_list(include_finished=true,newest_first=true,limit=3)`，首项仍为该任务，并返回非空下一页游标。
5. 只读检查产品数据库：首行名称为该任务、状态 `created`、创建时间为非零；其后迁移前任务创建时间为 `0`；`PRAGMA user_version=21`。

## 结论

macOS Verification Goal 达成。任务显示来自 SQLite/Gateway 权威快照，最新任务进入第一页顶部；协议兼容、迁移、稳定分页和 GUI 自动展开均有独立证据。Windows 证据继续暂缓，因此 DS-S2 完整跨平台 Story 仍保持 `verifying`，本增量不 Archive 为跨平台 Done。
