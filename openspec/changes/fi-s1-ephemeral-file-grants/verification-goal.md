# Verification Goal：FI-S1 临时文件授权引用核心

状态：PASS（Application/macOS File Core 子范围，2026-09-25）。产品原生选择器、Gateway、任务事件、可信覆盖/删除确认 UI 与 Windows 尚未接线，完整 FI-S1 保持 `implementing`，不 Archive/Done。

## 范围

- Story：FI-S1，FI-GRANT-01～05。
- OpenSpec：`fi-s1-ephemeral-file-grants`。
- 架构：Accepted AD-FI-01、AD-FI-02。
- 环境：Application 确定性合约与 macOS 临时目录；不访问用户文件、正式任务库或系统选择器。

## 验证矩阵

| 场景 | 可观察证据 | 结果 |
|---|---|---|
| 签发身份 | Agent 调用在 File Port 前返回 `PermissionDenied`；LocalUser 可签发 | PASS |
| 无正文记录 | 既有文件签发只保留规范路径、身份与 SHA-256，读取字节不进入授权记录 | PASS |
| 新目标预检 | 规范化既有父目录，返回父身份；链接逃逸与已存在目标拒绝且不创建文件 | PASS |
| 父目录绑定 | `CreateNew` 必须携带签发时父目录身份；身份不符返回 `IdentityChanged` 且无输出 | PASS |
| 会话隔离 | 当前 Agent、任务 owner、task_id、用途任一不符均拒绝，不能扩大授权 | PASS |
| 生命周期 | 最长 15 分钟；到期和显式撤销失效，新 Registry 不恢复旧授权 | PASS |
| 消费语义 | 读取在期限内复用；八线程并发解析写引用时恰好一个成功，其余 `NotFound` | PASS |
| Agent 撤权准备 | LocalUser 可按 owner 一次撤销该 Agent 全部授权 | PASS |
| 容量 | 256 项仍有效授权后新签发返回 `Capacity`，既有授权不被驱逐 | PASS |
| 全仓门禁 | Workspace、Python、Architecture、协议生成物及 71 个活动 OpenSpec 严格校验 | PASS |

结构化结果见 [`apps/desktop/evidence/fi-s1-file-grants-20260925/result.json`](../../../apps/desktop/evidence/fi-s1-file-grants-20260925/result.json)。本增量无 UI、无系统权限请求和外部协议，因此不生成截图。

## 命令与结果

- `cargo test --workspace --no-fail-fast`：131 项 Rust 测试通过（Adapters 63、Application 35、CLI 5、Desktop lib 7、Desktop bin 11、Domain 1、Protocol 9）。
- `python3 -m unittest discover -s scripts -p 'test_*.py' -v`：33 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `cargo run --locked -p yonder-protocol --example generate -- --check`：通过。
- `openspec validate --all --strict`：71 个活动 Change 与 10 个 Specs，共 81 项通过。
- 四个实现 Rust 文件定向 `rustfmt --check` 与 `git diff --check`：通过；`lib.rs` 只登记新模块。

## 范围限制

本 Goal 不证明用户已能从产品 UI 选择文件，不开放 Agent Gateway/CLI/MCP 文件方法，也不证明授权事实已进入任务事件/Outbox。授权引用不是命令风险确认，不能授权 Shell、提权、安装、删除、支付或发送。Windows File Adapter 继续返回 unavailable。
