# 接管定位历史

## ADDED Requirements

### Requirement: 定位阶段不可被最终结果覆盖

Yonder MUST 将已提交的定位开始和最终成功或失败分别保存并按原事件序号展示，不得从当前投影反推或覆盖历史。

#### Scenario: 定位成功

- **WHEN** 接管停止后提交 locating，并在核验后提交 focused
- **THEN** 历史分别显示正在定位和定位成功，二者绑定同一控制身份

#### Scenario: 定位失败

- **WHEN** 定位开始后以稳定失败分类结束
- **THEN** 历史保留 locating 和 failed 及其原因，任务仍保持暂停

#### Scenario: 旧库迁移

- **WHEN** schema 18 数据库升级且只存在最后的当前定位投影
- **THEN** Yonder 不补造旧定位阶段历史，既有任务、事件和当前投影保持不变

### Requirement: 定位历史保持协议和授权边界

Yonder MUST 只对协商协议 1.23 的授权读取输出 `focus_event`，并保持既有分页、连续性和编码预算。

#### Scenario: 旧协议或越权

- **WHEN** 1.22 客户端或非归属 Agent 查询同一任务
- **THEN** 旧协议不含新增字段，越权请求不泄露定位事实
