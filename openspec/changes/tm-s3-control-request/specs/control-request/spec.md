# 外部控制请求

## ADDED Requirements

### Requirement: pending不是停止
task.control成功只能表示控制已登记并正在停止；不得显示已接管、已定位或已记录。

### Requirement: 原子与幂等
pending控制、任务sequence、事件和Outbox必须同事务；相同控制重投幂等，不同控制冲突。

### Requirement: 授权与冻结
LocalUser可控制已有running任务，Agent仅所属；pending后不得派发新动作，只有匹配控制的边界停止事务才能完成并释放Permit。
