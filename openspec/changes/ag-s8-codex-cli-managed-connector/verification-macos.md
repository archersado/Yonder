# AG-S8 macOS Verification Goal

日期：2026-10-10
状态：真实连接样本通过；等待重启/不重放终局样本

## 目标

证明当前 macOS Codex CLI 会话可由受支持 `SessionStart` Hook 显式绑定到 Yonder；无需手填 thread ID 或 Socket，并在 Yonder UDS 重启后恢复同一 session，且不重放输入。

## 已完成证据

- 环境：macOS；Codex CLI 0.162.1；Yonder `dev` cb3cc88。
- 初始 `cargo test -p yonder-cli`：9/9 通过；该夹具把 proxy 错当作 JSONL，真实探针发现缺口后已由下述 10/10 分层合约替代。
- `codex plugin marketplace add` 与 `codex plugin add yonder-codex-connector@yonder-local` 成功；`codex plugin list --json` 显示插件 installed/enabled，安装缓存包含可执行连接脚本与 Hook 清单。
- macOS debug 应用包已重新生成；包内 `yonder codex-hook` 对非法 session 输入失败关闭，退出码 1、stdout 为空、诊断不含正文或完整 Payload。
- `openspec validate ag-s8-codex-cli-managed-connector --strict`、`python3 scripts/check_architecture.py`、JSON/Shell 语法和 `git diff --check` 通过。
- 兼容修复 `eedaf34` 后重新安装插件 `0.1.4`，在 Codex CLI `0.162.1 --no-daemon` 新会话中打开 `/hooks`：`SessionStart` 显示 Installed=2、Active=1、Review=1；待审项名称为“正在连接 Yonder”，来源为 `Plugin - yonder-codex-connector@yonder-local`，命令解析到安装缓存中的 `scripts/connect.sh`。
- 真实当前会话探针确认：把 proxy 当作 JSONL 会发生 initialize 超时；另起 `app-server --listen stdio://` 虽可 initialize，但 `thread/resume` 因当前会话已有 active writer 被拒绝。
- 修正后连接器在 `codex app-server proxy` 原始字节流上完成 WebSocket Upgrade，initialize 与当前 thread/resume 均通过；不设置 `YONDER_AGENT_ID` 时自动使用与正式 MCP 配置一致的 `codex-cli`，Gateway hello 通过并持续运行超过 12 秒确认窗口。进程树只保留一个连接器和一个受监管 proxy 子进程。
- `cargo test -p yonder-cli`：10/10 通过，新增原始 proxy 双向流、WebSocket App Server RPC 与 Gateway 重连分层合约。
- 真实重启恢复样本：保持原 `codex-hook` 与 proxy 进程不动，正常退出并重新启动正式 debug `Yonda.app`。桌面宿主 PID 发生变化；连接器在 300 秒恢复窗口内关闭旧连接，并建立指向新 Gateway 监听进程的全新 Unix Socket 对端。无需重启 Hook、重新填写 session/thread 或设置 `YONDER_AGENT_ID`。
- 不重放合约补强后 `cargo test -p yonder-cli` 11/11 通过：相同合法 input ID 连续进入处理两次，两次都返回既有确认，Codex App Server 只收到一次 `thread/read` 与一次 `turn/start`；输入正文未进入日志。
- Hook 生命周期清理样本：终止当前 Hook 宿主后，原连接器与其受监管 proxy 均在有界等待内消失，无残留；再次提交同一当前会话的受信 `SessionStart(resume)` 后，新连接器、单一 proxy 与新 Gateway Socket 均恢复连接。
- 圈选附件实现预检：Connector hello 已声明 `user_input_attachment`；17 项 CLI 单元/合约测试覆盖同会话 begin/chunk/finish、顺序/哈希/session/超限/过期、一次性消费、缺附件不降级文字、最大 4 MiB 内存图片帧和既有不重放；`node --check apps/desktop/ui/region-preview.js`、严格 OpenSpec 校验与 `git diff --check` 通过。
- 真实圈选附件样本：从 `dev` cb3cc88 重新构建并签名 debug `Yonda.app`，正常退出旧桌面宿主和旧 Connector 后，以当前受信 `SessionStart(resume)` 恢复同一 Codex session。Yonder 显示“任务通道已连接 · 输入通道已连接”；用户圈选无敏感内容的屏幕局部并提交“这是什么”，图片与文字作为同一条输入进入当前 Codex thread。accepted 后确认卡关闭、截图清场，Yonder 回到已连接的小龙窗口；未降级为纯文字，也未再出现“提交失败”。

## 待完成真实样本

Hook 已由用户审阅并信任，当前会话自动连接样本已通过。仍需完成：

1. 一次无副作用 replan 输入进入同一 Codex thread；

2026-10-10 尝试通过正式 Yonder MCP 创建无副作用 TextEdit Observe/replan 样本时，当前 Codex 会话内的 MCP stdio 已关闭，`task_create` 与 `task_list` 均返回 `Transport closed`。输入连接器仍保持连接；按产品边界不使用 shell 或直连 Gateway Socket 替代。须在下一个能重新拉起 Yonder MCP 的 Codex 会话中完成此样本。

2026-10-12 定位到阻断来自 AG-S1 `yonder mcp` 的10分钟业务空闲退出，而非本 Story 的输入连接器：Codex MCP宿主不会在下一次调用时重新拉起已退出进程。已同步修订 AD-AG-05、AG-S1 Story 与 `ag-s1-local-cli-mcp`，让MCP随stdio宿主存活；18项CLI回归通过。当前会话的transport已经关闭，不能由Yonder进程侧复活；必须在装入修正版CLI的新Codex MCP会话中完成下述无副作用replan样本，且仍不得使用Socket或SQLite替代。

## 当前结论

实现、隔离 macOS 合约、真实当前会话连接、Yonder 重启恢复、不重放、Hook/proxy 清理和真实圈选附件投递均通过；仅缺一次正式 MCP 触发的无副作用 replan 投递，生命周期 Goal 尚未 PASS。Story 保持 `verifying`，不得 Archive。Windows 与 ChatGPT Desktop 分别保持 Deferred/Unsupported。

## Hook 发现兼容性复核（2026-10-10）

- 本机路径实际调用的 Codex CLI `0.156.1` 可以安装并启用 portable 根清单，但其 `/hooks` 运行时未从根清单的 `extensions.com.openai.hooks` 发现 Yonder Hook。
- 插件仅通过 `.codex-plugin/plugin.json` 兼容清单内联声明 Hook。当前 Codex 存在 Agent Plugins 1.0 根清单分支不向 Hook runtime 传递 Hook source 的上游缺陷；根目录 `plugin.json` 会令兼容清单同样被跳过，因此在上游修复前不发布 portable 根清单。
- 插件版本提升至 `0.1.4`，确保本地市场重新安装时刷新缓存和 Hook 定义哈希。
- 验收方式：重新安装 `yonder-codex-connector@yonder-local`，新建 Codex CLI 会话，在 `/hooks` 的 `SessionStart` 详情中确认来源为 Yonder 插件并完成信任审阅。
