## ADDED Requirements

### Requirement: 本机用户结果确认
可信本机用户 SHALL 能够对终态任务提交一次结果确认，并且确认 SHALL NOT 改变任务执行终态或触发再次执行。

#### Scenario: 首次确认
- **WHEN** 本机用户确认一个终态任务
- **THEN** 系统记录确认、对应结果版本、产物清单版本和意见
- **AND** 任务执行状态保持不变

#### Scenario: 重复确认
- **WHEN** 用户重复提交同一确认标识
- **THEN** 系统返回既有确认回执，不追加历史事件

### Requirement: 不可变产物清单
产物集合 SHALL 以不可变清单版本发布，新增或变更产物 SHALL 生成新版本。

#### Scenario: 清单版本
- **WHEN** 任务新增、替换或失效产物
- **THEN** 系统创建新的 `manifest_version`
- **AND** 旧清单与旧确认保持不变

### Requirement: 审计容量门限
系统 SHALL 在写入前检查审计容量，容量不足时拒绝新创建、新执行或新确认。

#### Scenario: 容量不足
- **WHEN** 审计库达到 2 GiB 或可用磁盘低于 1 GiB
- **THEN** 系统拒绝新增写入并返回配额错误
- **AND** 不自动删除历史、用户产物或未同步/固定内容

### Requirement: 协议与存储兼容
协议 1.20 与 SQLite schema 16 SHALL 向后兼容，且迁移 SHALL 原子完成。

#### Scenario: 旧客户端
- **WHEN** 1.19 及以下客户端读取任务
- **THEN** 系统继续返回旧投影，不包含确认与清单字段

#### Scenario: 迁移失败
- **WHEN** schema 15→16 迁移失败
- **THEN** 系统整体回滚，不产生半提交状态
