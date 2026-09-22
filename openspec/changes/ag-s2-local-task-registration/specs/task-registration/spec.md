# 任务登记增量

关联AG-S2 AC1–7，AD-AG-02。

## ADDED Requirements

### Requirement: 已握手Agent登记任务

系统 MUST 只允许已握手且身份匹配的Agent登记任务，并在同一事务提交任务事实。

#### Scenario: 本地登记
- GIVEN 预绑定可信Agent会话协商1.1
- WHEN 请求task.create
- THEN SQLite同事务提交created任务、事件、Outbox与幂等记录，返回真实snapshot。

#### Scenario: 重试与冲突
- WHEN 同Agent同key同description重试
- THEN 返回同task_id当前快照，不增加事件。
- WHEN 同key不同description
- THEN -32009，不写库。

#### Scenario: 门禁
- WHEN 未握手、1.0、身份不符、LocalUser或过期请求创建
- THEN 拒绝且不写库；1.0查询继续兼容。

#### Scenario: 本地联调
- WHEN 独立测试Agent通过私有stdio发请求
- THEN 进入真实Gateway与SQLite，不人工插库，不伪称当前桌面进程已接入。

## Agent命名增量（2026-09-14）

关联AG-S2、TM-S1、DS-S2三份设计与Accepted AD-TM-07名称技术定稿。Architecture Impact：architecture-change（协议1.3、SQLite schema5）。只落实名称，不授权接管目标或Recording。

要求：1.3 Agent创建必须提供合法name；旧版不接收name且响应不增加name字段；新版本创建/get/list/cancel返回同一名称。名称1–256 UTF-8字节、非空、无控制字符，原样保存。相同owner/key仅在description和name完全相同才返回既有最新快照，否则-32009，不产生记录。取消后重试不恢复任务。

迁移前唯一SQLite备份，事务新增tasks.name与task_creations.name；旧NULL不伪造名称，保留所有业务记录。未知格式、已有加密旧版本、备份或DDL失败拒绝升级。卡片与详情用名称/历史缺名反馈，ID仍可查看。验证名称边界、旧新版门禁、原子回滚、备份与数据保留、正式嵌入UI和macOS原生显示；Windows暂缓、完整Story不归档。
