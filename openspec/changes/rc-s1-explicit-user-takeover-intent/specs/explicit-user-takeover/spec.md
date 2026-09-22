# 显式用户接管意图 Delta

## ADDED Requirements

### Requirement: 可信入口
系统 MUST 只把打包Task Space窗口调用专用本地命令视为显式用户接管意图；Agent、CLI、Gateway、普通输入和通用JSON查询 MUST NOT 建立用户控制租约。

#### Scenario: 非可信入口拒绝

- **WHEN** Agent、CLI、Gateway或普通输入请求建立用户控制租约
- **THEN** 请求拒绝，不创建用户接管事实

### Requirement: 参数收敛
UI MUST 只提交`task_id/expected_sequence`；kind、身份、能力、截止时间与请求身份 MUST 由可信Rust宿主生成。

#### Scenario: 可信宿主补齐请求

- **WHEN** 打包Task Space提交`task_id/expected_sequence`
- **THEN** Rust宿主生成kind、身份、能力和截止时间，不信任调用方自报字段

### Requirement: 复用既有接管事务
专用入口 MUST 复用TM-S3停止、定位和失败事实，不得创建第二套接管状态机；本增量 MUST NOT 启动Recording。

#### Scenario: 复用TM-S3事务

- **WHEN** 可信入口建立用户接管
- **THEN** 停止、定位和失败事实复用TM-S3事务，不启动Recording
