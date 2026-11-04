# TM-S6 全局运行态与收起预约 Windows Verification Goal

日期：2026-11-01
结论：Windows 核心平台与桌面消费者长时原生验证 PASS；PR 与 Archive 未完成。

## 结果

- Adapters 35/35、Application 19/19 Windows 测试通过。
- `global_running_state_ignores_pages_and_owners_and_fails_unknown`：超过 100 个前序任务及其他 owner 的 running 仍返回 Running；完成/恢复后返回 NoRunningTask；真实读取失败返回 Unknown；查询不改变 sequence、事件或 Outbox。
- `activity_includes_unreleased_background_work_and_unknown_sources`：数据库无 running 但准入占用未释放时仍 Busy；数据库失败且无占用为 Unknown，有占用时 Busy；丢弃凭证不清除占用。
- `poisoned_occupancy_is_unavailable_not_empty`：合成 Mutex 中毒按预期产生 panic 日志并被测试线程验证为 Unavailable，整个测试 PASS。
- `rest_reservation_checks_storage_and_blocks_start_without_writing` 与 `rest_and_execution_compete_atomically_and_dropped_rest_stays_reserved`：收起预约与执行准入互斥；读失败撤销未交付预约；预约期间不写任务；显式展开后恢复准入。

测试使用 Windows 临时 SQLCipher/SQLite 数据库。错误密钥 HMAC 日志是预期拒绝样本；LNK4099 缺少 OpenSSL PDB 只影响调试符号。

## 边界

桌面消费者另以真实BUA跨越180秒阈值验证：running期间保持展开，完成后重新进入自然收起并成功停靠，见 `verification-windows-native-dock-20261101.md`。PR 隔离审阅和 Archive 继续保留。
