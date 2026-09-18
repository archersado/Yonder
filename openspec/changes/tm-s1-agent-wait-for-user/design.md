# 设计

归属 Agent 在动作已 Observe 且 `task.step.advance` 形成 stopped 边界后提交等待请求。Application 校验身份、CAS、running 状态、无 pending control 与 stopped attempt；Adapter 在一个 IMMEDIATE 事务中更新 tasks、插入带 `wait_reason` 的事件和 Outbox。事务成功后 Admission 才释放任务占用。

`reason` 去除首尾空白后必须为 1～512 UTF-8 字节，日志不得记录正文。重复旧序号返回冲突，不以文本做幂等。协议 1.16 及更早版本不能调用，读取事件时也看不到新增字段。
