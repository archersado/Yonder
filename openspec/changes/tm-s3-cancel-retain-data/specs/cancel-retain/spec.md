# 取消保留数据

## ADDED Requirements

### Requirement: 无数据清理入口
卡片取消只改变合法状态，全部保留任务和历史；不得发task.delete或清理说明/事件/Outbox/幂等。

### Requirement: 安全兼容
实验格式4仅无删除标记且全transition事件时同事务回3，全部记录和序号不变；带标记/坏格式拒绝不修改。不新建清理格式4。
