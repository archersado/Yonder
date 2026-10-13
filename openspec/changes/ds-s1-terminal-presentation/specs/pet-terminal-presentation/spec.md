# 桌宠终态展示增量规格

## ADDED Requirements

### Requirement: 真实终态一次性反馈

系统 MUST 只对 Gateway 写请求本次成功提交的终态播放一次桌宠反馈。

#### Scenario: 完成提交

- Given 归属 Agent 通过统一 Gateway 成功提交任务完成
- When 响应包含新的 completed 序号
- Then Yonda 播放一次 success，隐藏时主动唤醒，并在约 1.8 秒后向宿主读取并恢复最新真实派生状态

#### Scenario: 终态响应早于异步投影

- Given Runtime 已将任务终结，但终态响应附近取得的 SQLite 展示快照仍为旧的 executing 步骤
- When success 或 failed 动画结束
- Then Yonda 不恢复事件中的旧快照，而是读取 Runtime 优先的最新宿主状态并清除旧步骤

#### Scenario: 终态退出与新状态竞争

- Given 终态退出刷新尚未返回
- When 新的普通展示事件到达，或最新状态读取失败
- Then 新事件优先且旧刷新结果不得覆盖；读取失败时显示 unknown 并清除旧步骤

#### Scenario: 读取终态

- Given 数据库已有 completed 或 failed 任务
- When 启动恢复或 Agent 查询任务
- Then 不播放 success 或 failed

#### Scenario: 只读请求遇到滞后检查点

- Given Runtime 已发布终态而 SQLite 异步检查点仍暂时显示旧 running 步骤
- When Agent 或 Task Space 调用 task.get、task.list 或 task.events
- Then 只读响应不得发布普通展示事件，也不得把 Yonda 改回 executing

#### Scenario: Runtime 终态覆盖旧检查点

- Given 当前进程 Runtime 已进入 completed、failed 或 cancelled，而 SQLite 仍保留旧 running 步骤
- When Yonda 读取最新桌宠展示状态
- Then 旧步骤被立即屏蔽，展示事件在原生 UI 主线程生效，且不等待 SQLite 投影完成

#### Scenario: 重复与失败响应

- Given 相同 task_id、sequence、status 的脉冲已经处理，或写请求被拒绝
- When 相同信息再次到达
- Then 不重播、不重复唤醒，且拒绝响应不伪造终态
