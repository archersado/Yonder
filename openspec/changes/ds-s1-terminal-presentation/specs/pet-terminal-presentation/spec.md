# 桌宠终态展示增量规格

## ADDED Requirements

### Requirement: 真实终态一次性反馈

系统 MUST 只对 Gateway 写请求本次成功提交的终态播放一次桌宠反馈。

#### Scenario: 完成提交

- Given 归属 Agent 通过统一 Gateway 成功提交任务完成
- When 响应包含新的 completed 序号
- Then Yonda 播放一次 success，隐藏时主动唤醒，并在约 1.8 秒后恢复真实派生状态

#### Scenario: 读取终态

- Given 数据库已有 completed 或 failed 任务
- When 启动恢复或 Agent 查询任务
- Then 不播放 success 或 failed

#### Scenario: 重复与失败响应

- Given 相同 task_id、sequence、status 的脉冲已经处理，或写请求被拒绝
- When 相同信息再次到达
- Then 不重播、不重复唤醒，且拒绝响应不伪造终态
