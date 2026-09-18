# TM-S6 收起预约独立 Verification Goal

Story: TM-S6
OpenSpec: tm-s6-running-state-query
日期：2026-09-12
依据：AD-TM-02 收起预约补充，AD-OCT-06。
状态：本地核心通过，未 Archive。

## 验证目标与环境

实施完成后核对 AC9–11，不再修改实现。macOS 本地、合成 SQLCipher v2 临时库；无真实用户数据、密钥提供器、原生窗口或外部发送。

## 结果

`/Users/archersado/.cargo/bin/cargo test --workspace --offline --locked` 退出 0，共 20 项通过（Adapter 9、Application 6、Domain 1、Protocol 4）。

- AC9：`rest_and_execution_compete_atomically_and_dropped_rest_stays_reserved` 使用两个线程、屏障同步并重复 32 轮；申请完成前不释放，验证恰好一方成功、预约成功拒绝重复预约及新执行，解除后正常准入。
- AC10：`rest_reservation_checks_storage_and_blocks_start_without_writing` 使用真实 SQLCipher 验证持久化 running、不可读表和缺失实例拒绝预约；数据库否决后可再次准入，证明未展示预约已撤销。锁中毒测试也确认预约返回 Unavailable。
- AC11：预约期间 start 返回 PresentationBusy，任务仍为 created，事件与 Outbox 各 1 条。确认展开后显式释放，start 成功。丢弃预约凭证后仍拒绝新准入。读取核对失败只撤销本次尚未交付的预约，不重试动作。
- 代码审阅：预约登记与 try_acquire 共用原 Mutex，数据库操作在锁外；RestPermit 不可克隆，没有 Drop 自动释放。全部真实执行走同一 Admission 是必需的宿主前置条件。
- 架构依赖、规划及本验证文件的合成 PR 关联检查通过。未创建实际 PR。协议生成一致性通过；既有 ts-rs 属性解析警告未变。

## 实际完成边界

查询快照仍不授予收起许可；新增预约只保证与同一 Admission 的执行准入互斥。DS 尚未持有该凭证驱动收起/展开，尚未实现新任务到来时唤醒再准入的宿主流程。没有本轮 Windows 或原生 UI 验证，不将核心通过等同桌面完成。

DS-S1 的 AD-E0-01 仍待定，正式宿主接线需先关闭基础栈验证门禁。当前工作区混有其他 Story 未提交改动，需隔离后审阅交付。TM-S6 保持 verifying。
