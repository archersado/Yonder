## ADDED Requirements

### Requirement: 已提交任务时间线
可信本机任务详情 SHALL 使用现有 `task.events` 展示已提交事件，不从当前快照反推历史。

#### Scenario: 步骤声明与执行结果
- **WHEN** 时间线包含步骤声明、observed 结果和 unknown 结果
- **THEN** 分别显示声明、已观察结论和未知原因，不把声明显示为已执行

#### Scenario: 局部读取失败
- **WHEN** 任务详情成功而事件查询失败
- **THEN** 保留任务详情并只在时间线区域显示读取失败

#### Scenario: 首批记录未完整
- **WHEN** 最后返回事件序号小于任务当前序号
- **THEN** 明确提示仍有记录未加载，不宣称时间线完整

#### Scenario: 空时间线
- **WHEN** 查询成功且没有事件
- **THEN** 显示暂无已提交记录，不显示读取失败
