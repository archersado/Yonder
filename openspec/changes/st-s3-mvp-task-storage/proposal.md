# 提案：MVP未加密任务存储

Story：ST-S3，docs/specs/epic-ST/story-ST-S3/。AD：Accepted AD-ST-01、AD-OCT-01。Architecture Impact：conforming。

按用户变更提供显式未加密SQLite打开方式，复用TaskStore、schema v2及事务；无协议、表结构或迁移变化，无新依赖。保留现有加密入口及证据，禁止自动回退、修改或覆盖旧加密库。三份设计已审阅，仅授权此独立存储子范围Apply。
