# TM-S2 尝试结果事务独立 Verification Goal

日期：2026-09-16。关联 TM-S2 RESULT-01～04、Accepted AD-TM-08/AD-CU-04 与 `openspec/changes/tm-s2-attempt-result/`。验证者在实现完成后运行锁定离线回归、真实 macOS 隔离动作和只读正式库迁移核对；Windows按用户要求暂缓。

## 结果

PASS（attempt结果事务子范围）。工作区锁定离线回归36项通过；Adapter同一项集成测试覆盖 observed/unknown、相同结果幂等、不同结论拒绝、旧身份约束、Outbox故障全回滚，以及协议1.4隐藏/1.5展示 `attempt_result`。Rust生成的JSON Schema与TypeScript同步，架构关联检查和 `git diff --check` 通过。

[macOS原生证据](../../../apps/desktop/evidence/cua-dispatch-20260915/result.json)确认真实 prepared attempt 经产品 SDK Worker 完成后台输入与后置 Observe 后，分类结果写入临时 schema8 SQLite，原生控件匹配、Worker退出且执行占用仍保留；未启动上游App、截图或Recording。正式 Yonda 已以 PID72536 重启，[迁移证据](../../../apps/desktop/evidence/attempt-result-20260916/result.json)记录 schema7→8 前后一致保留14个任务、0个attempt，新增4个结果列，并生成一致迁移前备份。

## 完成边界

结果事实不会结束 attempt 或释放租约。TM-S3 步骤边界停止、控制与派发排序、WorkRef定位、接管、Recording、产品Node/SDK完整发布打包和Windows证据仍未完成；TM-S2完整Story保持verifying，不Archive。
