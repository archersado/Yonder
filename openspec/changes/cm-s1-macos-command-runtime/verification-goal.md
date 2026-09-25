# Verification Goal：CM-S1 macOS 结构化命令 Runtime

状态：PASS（macOS Runtime 子范围，2026-09-25）。Windows 按用户决定延期；Agent Gateway 风险确认协议尚未定案，完整 CM-S1 保持 `implementing`，不 Archive/Done。

## 范围

- Story：CM-S1，结构化 `program + args + cwd + env`、有界输出、超时/取消和完整进程组停止。
- OpenSpec：`cm-s1-macos-command-runtime`。
- 架构：Accepted AD-CM-01 的 macOS-only Runtime 子范围。
- 环境：macOS；测试进程只使用隔离的 `/tmp` 标记文件和短生命周期子进程，不读取任务库或用户文件。

## 验证矩阵

| 场景 | 可观察证据 | 结果 |
|---|---|---|
| 字面参数与空继承环境 | Shell 形态参数原样输出且未创建标记文件；子进程只看到显式 `YONDER_ONLY` | PASS |
| 正常、非零与启动失败 | `exit 0/1` 保持稳定分类；缺失路径、目录和不可执行文件分别拒绝或返回启动失败 | PASS |
| 超时与取消 | 父进程及后台后代均停止；结果分别为 `timed-out`、`cancelled` | PASS |
| stdout/stderr 超限 | 任一路只保留 64 KiB、标记截断并停止进程组 | PASS |
| 意外后台后代 | 父进程退出但后代仍存活时主动清理并返回 `unknown`，不误报成功 | PASS |
| 平台边界 | 非 macOS 编译路径只返回 `UnsupportedPlatform`，没有单进程 kill 或 Shell 兜底 | PASS（静态/编译边界） |
| 全仓回归与门禁 | Workspace、Python、Architecture、协议生成物及 68 个活动 OpenSpec 严格校验 | PASS |

结构化结果见 [`apps/desktop/evidence/cm-s1-command-runtime-macos-20260925/result.json`](../../../apps/desktop/evidence/cm-s1-command-runtime-macos-20260925/result.json)。本增量没有新增 UI 或系统权限请求；执行 Driver 的 macOS 行为以无正文结构化测试日志取证。Windows 路径未运行，明确记录为 deferred/unavailable。

## 命令与结果

- `cargo test --offline --locked --workspace`：117 项 Rust 测试通过（Adapters 54、Application 30、CLI 5、Desktop lib 7、Desktop bin 11、Domain 1、Protocol 9）。
- `python3 -m unittest discover -s scripts -p 'test_*.py'`：33 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `cargo run --offline --locked -p yonder-protocol --example generate -- --check`：通过。
- 活动 OpenSpec `--strict`：68 个通过。
- `git diff --check`：通过。

## 范围限制

本 Goal 不证明 Windows Job Object 路线、Agent Gateway/CLI/MCP、Shell、风险确认 UI、任务状态/事件接线或 Jev Command 候选已完成。Application Port 只能由后续可信上层在完成授权与确认后调用；当前没有外部调用入口。
