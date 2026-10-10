# AG-S8 macOS Verification Goal

日期：2026-10-10
状态：等待一次性 Hook 信任后的真实会话样本

## 目标

证明当前 macOS Codex CLI 会话可由受支持 `SessionStart` Hook 显式绑定到 Yonder；无需手填 thread ID 或 Socket，并在 Yonder UDS 重启后恢复同一 session，且不重放输入。

## 已完成证据

- 环境：macOS；Codex CLI 0.162.1；Yonder `dev` 5a6df2d。
- `cargo test -p yonder-cli`：9/9 通过，覆盖 Hook 输入门禁、公开 daemon/proxy initialize + thread/resume、idle turn/start、去重上限、退避上限和 Yonder UDS 断开后的第二次同 session hello。
- `codex plugin marketplace add` 与 `codex plugin add yonder-codex-connector@yonder-local` 成功；`codex plugin list --json` 显示插件 installed/enabled，安装缓存包含可执行连接脚本与 Hook 清单。
- macOS debug 应用包已重新生成；包内 `yonder codex-hook` 对非法 session 输入失败关闭，退出码 1、stdout 为空、诊断不含正文或完整 Payload。
- `openspec validate ag-s8-codex-cli-managed-connector --strict`、`python3 scripts/check_architecture.py`、JSON/Shell 语法和 `git diff --check` 通过。

## 待完成真实样本

Codex 官方要求非托管 Hook 先由用户审阅并信任。不得使用 `--dangerously-bypass-hook-trust` 代替。用户需在 Codex `/hooks` 信任 `yonder-codex-connector`，重启 Yonder 后新建或恢复一个 Codex CLI 会话，再验证：

1. Yonder 自动显示输入通道已连接，无 thread/socket 环境变量；
2. 一次无副作用 replan 输入进入同一 Codex thread；
3. 重启 Yonder，五分钟内输入通道恢复；
4. 相同 input ID 与断开时未确认输入不向 Codex 重放；
5. 会话结束后连接器退出，无 proxy/bridge 残留。

## 当前结论

实现和隔离 macOS 合约通过；因官方 Hook 信任必须由用户显式完成，真实会话生命周期 Goal 尚未 PASS。Story 保持 `verifying`，不得 Archive。Windows 与 ChatGPT Desktop 分别保持 Deferred/Unsupported。

## Hook 发现兼容性复核（2026-10-10）

- 本机路径实际调用的 Codex CLI `0.156.1` 可以安装并启用 portable 根清单，但其 `/hooks` 运行时未从根清单的 `extensions.com.openai.hooks` 发现 Yonder Hook。
- 插件通过 `.codex-plugin/plugin.json` 兼容清单唯一声明 `./hooks/hooks.json`；portable 根清单只保留跨宿主产品元数据，避免其中的 `extensions.com.openai` 覆盖整个兼容清单。
- 插件版本提升至 `0.1.2`，确保本地市场重新安装时刷新缓存和 Hook 定义哈希。
- 验收方式：重新安装 `yonder-codex-connector@yonder-local`，新建 Codex CLI 会话，在 `/hooks` 的 `SessionStart` 详情中确认来源为 Yonder 插件并完成信任审阅。
