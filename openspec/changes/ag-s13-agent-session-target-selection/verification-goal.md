# AG-S13 独立 Verification Goal

日期：2026-10-14。平台：macOS arm64。状态：PARTIAL；实现与自动化门禁通过，真实 Agent 回复朗读/停止样本待当前 MCP 客户端重连后复跑。Windows 按既有安排 Deferred。

## 目标

独立确认多 AgentSession 并存时 Yonder 不猜测接收目标，用户能从保留草稿的圈选卡直接打开 Agent 管理、选择一个短生命会话引用，并让后续输入只进入该会话；目标断连后必须重新收敛。选择状态不得持久化或暴露完整 session/thread ID。

## 已通过证据

- `cargo test -p yonder-desktop --locked`：lib 39/39、bin 11/11 通过；覆盖自动选择、显式选择、非目标上下线、目标断连、陈旧引用拒绝以及既有输入/回复回归。
- `node apps/desktop/check-agent-session-targeting.mjs`：通过；确认正式 Agent 管理列表/选择命令、连接变化刷新、圈选卡直达入口和选择完成恢复通知，且 UI 不读取完整 session ID。
- `check-region-submit-recovery.mjs`、`check-voice-reply.mjs`：通过；提交失败仍保留现场，语音回复显式开关和停止入口未回归。
- `openspec validate ag-s13-agent-session-target-selection --strict`、`python3 scripts/check_architecture.py`、`git diff --check`：通过。
- 从 `dev` 构建 `yonder-desktop` 与 `yonder-cli` 并生成正式预览包成功；重启后真实 PID 38618 的原生辅助功能树显示“任务通道已连接 · 输入通道已连接”。

## 待复跑项

正式宿主重启使本轮既有 MCP stdio transport 返回 `Broken pipe`；虽然 Codex hook 已重连并在 Yonder 显示输入通道在线，但当前工具会话不能绕过 Gateway 重建连接。`cua_repl` 能打开真实圈选窗口，但对全屏透明 selector 的坐标动作被宿主拒绝，不能替代用户真实指针完成圈选。待当前客户端重连后按以下顺序复跑：

1. 圈选卡提交并触发多会话目标缺失。
2. 点击卡片内“选择接收会话”，在 Agent 管理选定当前 Codex 会话。
3. 返回原卡确认草稿/截图仍在，重新发送并确认只在目标会话收到输入。
4. 由真实 Agent 流式回复，观察“正在准备语音/正在朗读”，点击“停止朗读”，确认文字保留。

上述四步完成前，本 Goal 不记 PASS，不 Archive AG-S13，也不把自动化夹具描述为正式产品 E2E。
