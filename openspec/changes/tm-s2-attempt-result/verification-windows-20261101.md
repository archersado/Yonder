# TM-S2 尝试结果事务 Windows Verification Goal

日期：2026-11-01
结论：Windows 事务/协议与真实 BUA Driver 子范围 PASS；真实 Windows CUA Driver 不在本轮范围，完整 Story 不 Archive。

## 结果

- `cargo test --offline --locked -q -p yonder-adapters`：35/35 PASS。
- `cargo test --offline --locked -q -p yonder-application`：19/19 PASS。
- 定向测试 `execution_attempt_preparation_is_atomic_idempotent_and_step_bound` 在 Windows PASS：prepared attempt 原子绑定 running/事件/Outbox；observed 结果写入同一序号；相同结果幂等，不同结论和旧 Worker 身份冲突；协议 1.4 隐藏、1.5 展示 `attempt_result`。
- 同一 Adapter 回归包含 unknown 分类与 Outbox 注入失败全回滚。测试只使用临时库，不访问用户任务库。
- 正式 Windows BUA 任务另有 8 条真实 `ego_bridge` attempt（create、6轮Observe、finish），全部写入 `stopped`、`action_succeeded=1`、`observe_valid=1` 且无 unknown；任务最终 `completed@34`。证据与只读核对见 `../tm-s2-execution-attempt/verification-windows-bua-20261101.md`。

运行日志中的 SQLCipher HMAC 错误来自错误密钥拒绝测试，是预期安全失败；测试最终退出 0。Windows OpenSSL 静态库缺少调试 PDB 的 LNK4099 仅影响调试符号，不影响测试结果。

## 边界

结果事务自身不结束 attempt、不释放租约；真实 BUA Driver 已通过后续停止语义完成 attempt。本轮按用户要求不执行 CUA；CUA Driver、TM-S3 停止控制及完整 Story/PR/Archive 继续保留。
