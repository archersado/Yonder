# Verification Goal：FI-S1 macOS 受控文件 Runtime

状态：PASS（macOS Runtime 子范围，2026-09-25）。Windows 按用户决定延期；Agent Gateway 文件授权、任务生命周期和可信覆盖/删除确认尚未接线，完整 FI-S1 保持 `implementing`，不 Archive/Done。

## 范围

- Story：FI-S1，FI-01～06 的内部 macOS File Port/Adapter 子范围。
- OpenSpec：`fi-s1-macos-file-runtime`。
- 架构：Accepted AD-FI-01 的 macOS-only Runtime。
- 环境：macOS；全部读写与回收站样本使用唯一临时目录和自建文件，不访问用户文件或正式任务库。

## 验证矩阵

| 场景 | 可观察证据 | 结果 |
|---|---|---|
| 身份与授权根 | 原路径、软链接、硬链接返回同一 `st_dev + st_ino`；链接逃逸在读取前拒绝 | PASS |
| 有界读取 | 返回内容、身份和 SHA-256；超过 16 MiB 稳定拒绝 | PASS |
| 新建与替换 | 同目录独占暂存、校验、文件同步、原子提交和父目录同步；替换产生新身份 | PASS |
| 陈旧与竞争 | 旧身份、旧哈希、并发出现目标分别拒绝，竞争方内容不被覆盖 | PASS |
| 双层锁 | 同一身份第二写租约返回 busy；外部 advisory lock 返回 host-locked | PASS |
| 失败清理 | 校验失败保持原文件且无暂存残留；rename 后父目录同步故障返回 unknown 并保留可能已提交结果 | PASS |
| 系统回收站 | Agent 在 Adapter 前拒绝；完整身份匹配后用 `NSFileManager.trashItem` 移动临时夹具，验证身份后清理夹具 | PASS |
| WPS 宿主锁路线 | 复用既有真实 WPS 打开拒绝、关闭后可锁证据；产品使用同一 `File::try_lock` | PASS（既有原生证据） |
| 平台边界 | 非 macOS 实现只返回 `UnsupportedPlatform`，没有直接删除或复制覆盖兜底 | PASS（静态/编译边界） |
| 全仓回归与门禁 | Workspace、Python、Architecture、协议生成物及 69 个活动 OpenSpec 严格校验 | PASS |

结构化结果见 [`apps/desktop/evidence/fi-s1-file-runtime-macos-20260925/result.json`](../../../apps/desktop/evidence/fi-s1-file-runtime-macos-20260925/result.json)。本增量没有 UI 或新增系统权限请求；File Driver 的 macOS 行为以无正文结构化测试日志及既有 WPS 原生锁证据取证。Windows 路径未运行，明确记录为 deferred/unavailable。

## 命令与结果

- `cargo test --offline --locked --workspace`：123 项 Rust 测试通过（Adapters 59、Application 31、CLI 5、Desktop lib 7、Desktop bin 11、Domain 1、Protocol 9）。
- `python3 -m unittest discover -s scripts -p 'test_*.py'`：33 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `cargo run --offline --locked -p yonder-protocol --example generate -- --check`：通过。
- 活动 OpenSpec `--strict`：69 个通过。
- 新增 Rust 文件定向 `rustfmt --check` 与 `git diff --check`：通过。

## 范围限制

本 Goal 不证明 Windows 文件身份/锁、Agent Gateway/CLI/MCP、任务 attempt/事件、授权文件选择、覆盖确认 UI、目录/批量操作或 DO-S2 文件接线已完成。Runtime 不提供永久删除，回收站 Port 还要求 Application 生成的不可构造本机授权令牌。
