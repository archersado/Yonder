# TM-S3 步骤边界停止独立 Verification Goal

日期：2026-09-16。关联 TM-S3 STOP-04～06、Accepted AD-CU-02/AD-TM-08 与 `openspec/changes/tm-s3-step-boundary-stop/`。验证者在实现完成后运行锁定离线回归及只读正式库迁移核对；本增量无新UI/Driver操作，Windows按用户要求暂缓。

## 结果

PASS（内部步骤边界停止子范围）。工作区锁定离线回归36项通过。Adapter同一集成测试覆盖：当前observed attempt接管转paused并提交stopped；取消转cancelled且保留数据；相同控制幂等；不同控制及旧Worker拒绝；unknown拒绝停止且Permit保持占用；Outbox故障使任务/事件/attempt全部回滚并交还Permit，修复后重试成功才释放。架构关联检查及 `git diff --check` 通过。

正式 Yonda 已重建并以 PID85873 单实例运行。[迁移证据](../../../apps/desktop/evidence/step-boundary-stop-20260916/result.json)记录正式未加密库 schema8→9，前后一致保留14个任务、0个attempt，新增3个停止字段并生成一致迁移前备份。没有为验证向正式库虚构运行、attempt或控制记录。

## 完成边界

本Goal只证明内部observed步骤边界事务和先提交后释放。外部Agent/本地UI控制协议、派发与控制真正并发排序、接管按钮接线、WorkRef窗口定位、Recording、交回恢复和Windows证据未完成；TM-S3保持implementing，不Archive。
