# TM-S6 执行占用汇总独立 Verification Goal

Story: TM-S6
OpenSpec: tm-s6-running-state-query
日期：2026-09-12
依据：AD-TM-02 补充决定、AD-OCT-06。
状态：本地核心检查通过；未 Archive。

## 目标

实施完成后独立核对 AC5–8。数据库无 running 不得覆盖未停止执行占用；异常不得伪装为无工作。验证阶段不修改实现。

## 环境与结果

macOS 本地，合成 SQLCipher v2 临时库及合成子密钥，无用户数据或系统凭据访问。

`/Users/archersado/.cargo/bin/cargo test --workspace --offline --locked`：退出 0，18 项通过（Adapter 8、Application 5、Domain 1、Protocol 4）。既有 ts-rs 属性解析警告保留；生成协议一致性检查通过。

- `activity_includes_unreleased_background_work_and_unknown_sources`：真实准入 start 空资源后台任务；终态提交后仍 Busy，确认停止并释放后为 NoKnownWork；无占用但有持久化 running 仍 Busy（AC5–6）。
- 同一测试：未提供已恢复实例为 Unknown；数据库读失败且无占用为 Unknown；读失败但有占用为 Busy；丢弃凭证后、数据库恢复可读仍 Busy（AC6–7）。
- `poisoned_occupancy_is_unavailable_not_empty`：真实 Mutex 中毒返回 Unavailable；汇总 match 对非忙碌错误返回 Unknown，代码审阅确认没有 unwrap 或空值降级（AC7）。
- 集成测试核对只读汇总不改任务 sequence、事件及 Outbox，不释放占用。代码审阅确认数据库查询在占用锁之外，无缓存与协议扩展（AC8）。
- `scripts/check_architecture.py`：依赖与规划门禁通过；本轮独立记录通过合成 PR 正文的 Story/OpenSpec/Verification 关联检查，未实际创建 PR。

## 尚未完成

两次读取不是原子快照；调用方必须传恢复后的唯一 Admission，不能传临时空实例。尚无真实桌面接线、Windows 本轮运行证据、收起与新任务准入串行保证。当前工作区混有其他 Story 变更，未提交或创建 PR。TM-S6 保持 verifying，TM-S1 AC11 整体仍未关闭。
