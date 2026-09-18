# 用户任务删除

状态：Withdrawn。以下delta requirements已由Accepted AD-TM-06撤销，不进入有效规格或产品实现。

## ADDED Requirements

### Requirement: 确认与授权
仅LocalUser、宿主NoKnownWork、终态且期望序号匹配可删除；用户确认前不发请求，Agent/活动任务/未知占用拒绝。

### Requirement: 原子清理与防重放
说明/旧事件/旧Outbox同事务清理，保留最小deleted标记和新删除事件/Outbox。普通查询隐藏标记，创建重试-32013不复活，重复删除同前/当前序号不增事件。

### Requirement: 迁移与证据
2/3到4迁移同事务保留数据；错误版本拒绝。实际清理使用独立测试库，原生真实任务仅打开确认后取消，Windows完成门禁保留。
