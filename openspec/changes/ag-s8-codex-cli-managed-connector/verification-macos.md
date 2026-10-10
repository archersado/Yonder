# AG-S8 macOS Verification Goal

日期：2026-10-10
状态：真实连接样本通过；等待重启/不重放终局样本

## 目标

证明当前 macOS Codex CLI 会话可由受支持 `SessionStart` Hook 显式绑定到 Yonder；无需手填 thread ID 或 Socket，并在 Yonder UDS 重启后恢复同一 session，且不重放输入。

## 已完成证据

- 环境：macOS；Codex CLI 0.162.1；Yonder `dev` 5a6df2d。
- 初始 `cargo test -p yonder-cli`：9/9 通过；该夹具把 proxy 错当作 JSONL，真实探针发现缺口后已由下述 10/10 分层合约替代。
- `codex plugin marketplace add` 与 `codex plugin add yonder-codex-connector@yonder-local` 成功；`codex plugin list --json` 显示插件 installed/enabled，安装缓存包含可执行连接脚本与 Hook 清单。
- macOS debug 应用包已重新生成；包内 `yonder codex-hook` 对非法 session 输入失败关闭，退出码 1、stdout 为空、诊断不含正文或完整 Payload。
- `openspec validate ag-s8-codex-cli-managed-connector --strict`、`python3 scripts/check_architecture.py`、JSON/Shell 语法和 `git diff --check` 通过。
- 兼容修复 `eedaf34` 后重新安装插件 `0.1.4`，在 Codex CLI `0.162.1 --no-daemon` 新会话中打开 `/hooks`：`SessionStart` 显示 Installed=2、Active=1、Review=1；待审项名称为“正在连接 Yonder”，来源为 `Plugin - yonder-codex-connector@yonder-local`，命令解析到安装缓存中的 `scripts/connect.sh`。
- 真实当前会话探针确认：把 proxy 当作 JSONL 会发生 initialize 超时；另起 `app-server --listen stdio://` 虽可 initialize，但 `thread/resume` 因当前会话已有 active writer 被拒绝。
- 修正后连接器在 `codex app-server proxy` 原始字节流上完成 WebSocket Upgrade，initialize 与当前 thread/resume 均通过；不设置 `YONDER_AGENT_ID` 时自动使用与正式 MCP 配置一致的 `codex-cli`，Gateway hello 通过并持续运行超过 12 秒确认窗口。进程树只保留一个连接器和一个受监管 proxy 子进程。
- `cargo test -p yonder-cli`：10/10 通过，新增原始 proxy 双向流、WebSocket App Server RPC 与 Gateway 重连分层合约。

## 待完成真实样本

Hook 已由用户审阅并信任，当前会话自动连接样本已通过。仍需完成：

1. 一次无副作用 replan 输入进入同一 Codex thread；
2. 重启 Yonder，五分钟内输入通道恢复；
3. 相同 input ID 与断开时未确认输入不向 Codex 重放；
4. 会话结束后连接器退出，无 proxy/bridge 残留。

## 当前结论

实现、隔离 macOS 合约和真实当前会话连接通过；重启、不重放及会话结束清理样本尚未齐全，生命周期 Goal 尚未 PASS。Story 保持 `verifying`，不得 Archive。Windows 与 ChatGPT Desktop 分别保持 Deferred/Unsupported。

## Hook 发现兼容性复核（2026-10-10）

- 本机路径实际调用的 Codex CLI `0.156.1` 可以安装并启用 portable 根清单，但其 `/hooks` 运行时未从根清单的 `extensions.com.openai.hooks` 发现 Yonder Hook。
- 插件仅通过 `.codex-plugin/plugin.json` 兼容清单内联声明 Hook。当前 Codex 存在 Agent Plugins 1.0 根清单分支不向 Hook runtime 传递 Hook source 的上游缺陷；根目录 `plugin.json` 会令兼容清单同样被跳过，因此在上游修复前不发布 portable 根清单。
- 插件版本提升至 `0.1.4`，确保本地市场重新安装时刷新缓存和 Hook 定义哈希。
- 验收方式：重新安装 `yonder-codex-connector@yonder-local`，新建 Codex CLI 会话，在 `/hooks` 的 `SessionStart` 详情中确认来源为 Yonder 插件并完成信任审阅。
