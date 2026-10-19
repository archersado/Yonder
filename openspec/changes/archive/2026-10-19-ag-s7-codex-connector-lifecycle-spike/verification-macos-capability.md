# AG-S7 macOS Codex 连接生命周期能力探针

Story: AG-S7  
Date: 2026-10-10  
Result: PASS

## 范围

本证据只验证当前 macOS 主机的公开 CLI 能力、显式绑定失败关闭和 ChatGPT Desktop 进程公开传输形态；不访问真实 Codex thread，不发送模型请求，不宣称产品生命周期已经完成。

## 命令

```bash
cargo build -p yonder-cli
python3 apps/yonder-cli/check-codex-connector-lifecycle-macos.py
```

## 结果

- 当前 Codex CLI 版本满足 AG-S5 的最低版本，并公开 `app-server --listen unix://...`。
- `codex resume --remote unix://...` 可用，支持明确恢复到共享 App Server。
- 未提供 thread ID 时 `agent-bridge` 稳定失败关闭，不猜测最近会话。
- 已提供 thread ID 但未提供 endpoint 时稳定失败关闭，不附着私有 stdio。
- 当前 ChatGPT Desktop 样本存在 Codex App Server 子进程，但未公开 Unix/WebSocket endpoint；该样本只支持记录为 Unsupported/Deferred，不能外推 CLI 能力。

## 未覆盖

- 真实 remote TUI 的冷启动、Yonder 重启、桥崩溃、Codex 重启与 thread 切换连续样本；
- Windows 对照；
- 后续产品守护进程与安装生命周期。
