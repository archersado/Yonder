# Verification Goal：FI-S1 macOS Agent 受控文件执行

状态：通过（macOS）；Windows 按用户决定延期，未计为通过。

## 覆盖范围

验证协议 1.28 的受授权文件执行闭环：请求不携带位置事实；Gateway 仅为已认证归属 Agent 调用 TaskHost 的 Registry 与 File Port；替换授权一次性消费，并在 SQLite 任务执行事实下产生可观察结果。大文件传输、目录操作、永久删除与 Windows 不在本 Goal 范围内。

## 证据（2026-09-26）

| 检查 | 结果 | 结论 |
| --- | --- | --- |
| `cargo test -p yonder-desktop --lib tests::file_execute_consumes_replace_grant_without_returning_a_path -- --exact` | 1 通过 | macOS TaskHost/Gateway 使用替换授权修改测试文件；响应不含文件名或父目录；授权执行后不再可列出。 |
| `cargo test -p yonder-protocol tests::file_execute_contract_is_bounded_and_has_no_location_fields -- --exact` | 1 通过 | 严格拒绝位置字段、非法 Base64 与超过 48 KiB 的编码。 |
| `cargo test -p yonder-protocol -p yonder-application -p yonder-desktop -p yonder-cli` | 71 通过 | 协议、Application、桌面宿主与 MCP 类型请求回归。 |
| 协议生成、OpenSpec、架构及发布契约门禁 | 通过 | Rust 唯一协议源生成物、Change、架构关联与发布版本一致。 |

测试数据仅在临时目录创建，正文和路径均未记录到证据。真实文件副作用仍使用现有 File Port 的身份、哈希、锁和原子提交保护；unknown 不会自动重试。

## 后续

- Windows Runtime 与原生证据仍为延期项，不标记为 Windows PASS。
- 大文件附件/分片传输、目录操作、永久删除与 OOXML 语义变换继续由后续独立 Change 定案。
- FI-S1 整体仍为 `implementing`，本 Goal 只关闭该 macOS Agent 执行增量。
