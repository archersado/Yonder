# goal-verification Delta Specification

## MODIFIED Requirements

### Requirement: 用户目标只能由归属慢脑基于最新 Observation 核验

系统必须区分 Driver 动作状态、计划片段推进和用户目标完成。Application 只消费 Driver 直接返回的结构事实，不得根据动作类型合成业务状态；用户目标完成只接受归属慢脑绑定最新 Observation 的 `task.goal.verify`。

#### Scenario: 播放动作返回后由慢脑核验

- **WHEN** Driver 对播放相关动作返回动作状态及最新 Observation
- **THEN** Application 只报告动作是否明确成功、Observe 是否有效和目标是否可见
- **AND** 不把 `activate-control` 推导成歌曲正在播放或目标已完成
- **AND** 归属慢脑读取最新 Observation 后提交 achieved/not-achieved

#### Scenario: 业务寓意条件不再合成

- **WHEN** 旧计划以 target-resolved、composer-ready 或 delivery-confirmed 作为后置条件
- **THEN** Application 不根据动作种类构造这些真值
- **AND** 缺少 Driver 可证明状态时安全交回慢脑

#### Scenario: 结构事实允许片段继续

- **WHEN** Driver 明确返回动作成功、Observe 有效及可选目标可见状态
- **THEN** Application 可依据对应结构条件继续下一已授权槽位
- **AND** 片段推进不得替代最终目标核验
