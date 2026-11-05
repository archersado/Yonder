# agent-desktop Driver Spike Delta Spec

## ADDED Requirements

### Requirement: 候选 Driver 必须隔离在 Yonder 监督边界内

候选引擎 MUST 只运行于受监管独立 Worker，且 MUST NOT 成为任务状态、派发准入或协议所有者。

#### Scenario: 启动候选 Worker

- **WHEN** Spike harness 启动固定 revision 的候选引擎
- **THEN** 固定 revision 的最小 Rust bridge 只运行在独立 Worker
- **AND** 通信只使用 `agent-desktop-spike-bridge/v1` JSONL over inherited stdio
- **AND** 不使用 status-only C ABI、CLI 或其他私有 IPC
- **AND** 不开放 HTTP/TCP、不调用上游 MCP、不修改产品 Gateway 或 SQLite

#### Scenario: Bridge 握手

- **WHEN** Worker 建立 inherited stdio JSONL 会话
- **THEN** `hello_result` 必须回显固定 source revision、bridge contract hash、sample definitions hash 与 capabilities hash
- **AND** 任一版本、hash、attempt/Worker/fence 身份不匹配时 Host 必须禁止派发
- **AND** bridge 的 Rust 类型必须生成并校验 JSON Schema

#### Scenario: Driver 模型泄漏

- **WHEN** 上游命令、snapshot ref、lease 或 disposition 需要进入产品边界
- **THEN** Adapter 必须把它们收敛为既有 ComputerUse Port 语义
- **AND** Domain、Gateway 与持久化不得新增第二套上游模型

### Requirement: 只允许可信目标动作

Spike MUST 只开放 snapshot、qualified ref 与经验证窗口操作；目标不确定时 MUST fail closed。

#### Scenario: 目标身份有效

- **WHEN** 当前 snapshot/ref、平台窗口身份、fencing 与 input generation 全部匹配
- **THEN** harness 可以向 Worker 派发一个 allowlist 动作
- **AND** 动作后必须执行新鲜 Observe

#### Scenario: 目标身份漂移

- **WHEN** 出现同名歧义、应用重启、句柄复用、stale ref 或 hit-test unknown
- **THEN** harness 必须拒绝动作或返回 unknown
- **AND** 不得改选同名窗口、raw 坐标或当前前台队列

#### Scenario: 被禁能力

- **WHEN** 请求 raw 坐标、Windows 无目标按键、`--force`、CDP 或上游 MCP，或未显式关闭默认 trace/启用了 screenshots
- **THEN** harness 必须在调用 Driver 前拒绝

### Requirement: 人类输入必须抢占后续派发

平台观察器 MUST 在不记录输入正文的前提下区分真实用户输入与候选 Driver 合成输入，并使真实用户输入冻结后续动作。

#### Scenario: 用户在动作之间输入

- **WHEN** 真实用户输入使 input generation 变化
- **THEN** Application 监督语义必须先冻结下一次派发
- **AND** 此后不得向 Driver 交付新动作
- **AND** 证据不得包含按键、文本或轨迹正文

#### Scenario: 输入来源封闭分类

- **WHEN** 观察器收到不能同时绑定当前 `attempt_id` 与 `worker_instance_id` 的事件
- **THEN** 事件必须分类为 `user`、`unknown` 或 `observer_lost`
- **AND** generation 必须递增并冻结派发
- **AND** 只有 `driver_current_attempt` 可以不触发冻结

#### Scenario: Observer 未就绪或重启

- **WHEN** observer 未 ready、丢失、重启或 generation 回退
- **THEN** harness 必须 fail closed 并拒绝新派发
- **AND** Worker 重启不得重置宿主会话级 `u64 input_generation`

#### Scenario: 无法区分输入来源

- **WHEN** 平台无法可靠区分用户输入与 Driver 合成输入
- **THEN** Spike 必须判定候选路线淘汰
- **AND** 不得通过关闭用户抢占要求继续验证

### Requirement: 取消必须按交付边界分类

未派发及步骤间取消 MAY 确定完成；在途动作终止后只要副作用可能交付，结果 MUST 为 `unknown` 且 MUST NOT 自动重试。

#### Scenario: 唯一派发线性化点

- **WHEN** Supervisor 准备交付绑定当前 attempt/Worker 的动作
- **THEN** 派发、用户抢占和取消必须共用 Application-owned admission gate
- **AND** 完整 JSONL 动作帧写入并 flush inherited stdio 是唯一线性化点
- **AND** flush 前为 `not_dispatched`，flush 后为 `in_flight`

#### Scenario: 未派发取消

- **WHEN** 取消在线性化派发点之前生效
- **THEN** harness 确认没有动作交付并停止后续派发

#### Scenario: 在途终止

- **WHEN** 动作已经进入 Driver 且 Supervisor 终止 Worker
- **THEN** 结果为 `unknown`
- **AND** 不报告副作用已回滚、不自动重试
- **AND** 只允许资源检查、新鲜 Observe 和交回外部 Agent

#### Scenario: Worker 清理不完整

- **WHEN** Worker 退出后 interaction lease、进程树、按键/鼠标状态或临时资源仍有残留或无法证明释放
- **THEN** 样本失败并淘汰候选路线
- **AND** 进入 global desktop quarantine，不启动下一 Worker 执行动作
- **AND** 不得发送合成 key-up/button-up 冒充清理

#### Scenario: Quarantine 解除

- **WHEN** 全局桌面处于 quarantine
- **THEN** 只有宿主重启或明确只读恢复程序证明进程树、lease、输入状态、临时资源、observer 和新鲜 Observe 全部正常后才可重新准入

### Requirement: 每个动作必须由 Observe 闭环

Driver 返回 MUST NOT 单独成为成功事实；只有新鲜 Observe 验证预期条件时才可报告已知成功。

#### Scenario: Observe 验证成功

- **WHEN** 动作返回且新 snapshot 保持相同目标身份并验证预期条件
- **THEN** harness 记录有界已知结果

#### Scenario: Predicate 与身份判定被冻结

- **WHEN** 任一样本开始执行
- **THEN** sample catalog 必须已定义 `predicate_id`、无敏感正文的预期条件和只读 target identity oracle
- **AND** 平台实现不得自行改变通过谓词

#### Scenario: 交付或 Observe 不明

- **WHEN** disposition 为 `delivered_unverified`、Observe 失败、Worker 断连、deadline 或响应身份不一致
- **THEN** 结果为 `unknown`
- **AND** 不自动重发动作

### Requirement: 双平台证据必须使用相同硬门槛

Windows 与 macOS MUST 使用相同样本 ID、判定字段和零容忍安全门槛，并分别提交原生证据。

#### Scenario: 证据 schema

- **WHEN** 任一样本写入结构化证据
- **THEN** 证据必须符合 `agent-desktop-spike-evidence/v1`
- **AND** 包含 build manifest、平台/架构、sample/predicate、attempt/worker/fence、generation、dispatch、delivery、Observe、cleanup、单调耗时和 outcome
- **AND** 不包含输入正文、截图、完整窗口树、应用列表或完整 Payload

#### Scenario: 非法证据组合

- **WHEN** Observe identity/predicate 不通过、cleanup 为 residual/unproven 或 quarantine 为 true
- **THEN** `evidence-schema-v1.json` 必须拒绝把该记录表示为安全成功
- **AND** retry count 必须固定为 0

#### Scenario: 固定应用任务集

- **WHEN** 运行 CUA Driver 升级矩阵
- **THEN** Windows 必须覆盖 WPS 与文件资源管理器，macOS 必须覆盖 Microsoft Office 与 Finder
- **AND** 环境缺失只能记为 `environment_unavailable` 并保持 Spike 未完成
- **AND** 已安装目标不支持时记为 `unsupported` 并淘汰候选

#### Scenario: 完成单平台

- **WHEN** 仅一个平台完成样本
- **THEN** AD-CU-07 保持 Proposed
- **AND** 不提出产品 Apply 或替换 trycua

#### Scenario: 任一硬门槛失败

- **WHEN** 任一平台发生误投递、用户输入后新增派发、unknown 自动重试、资源残留、未声明运行库或签名责任链不可建立
- **THEN** 候选路线必须淘汰
- **AND** 其他样本通过不得抵消该失败

#### Scenario: 双平台全部通过

- **WHEN** 两平台全部硬门槛和独立 Verification Goal 通过
- **THEN** 只更新 ADR 结论
- **AND** 产品接线仍需新的 Story 与 OpenSpec Apply Change

### Requirement: 可重现构建与平台分发责任链

Spike MUST 从固定源码和完整 build manifest 产生可复核制品，并分别验证来源证明、操作系统平台签名和干净机依赖。

#### Scenario: 固定构建输入

- **WHEN** Windows 或 macOS 构建 Worker bridge
- **THEN** build manifest 必须记录上游 commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`、patch hash、Cargo.lock hash、Rust toolchain、target triple、features、链接策略、bridge/evidence schema、harness commit/dirty diff 与构建命令
- **AND** 两个平台必须使用同一源码、patch set 和语义入口

#### Scenario: 干净 Windows 启动

- **WHEN** 在未预装开发工具和 VC++ Runtime 的受控干净 Windows 上启动制品
- **THEN** Worker 必须完成只读握手与 Observe
- **AND** 不得依赖未在分发清单声明的运行库

#### Scenario: SBOM 与第三方许可

- **WHEN** 构建完成
- **THEN** 必须生成与制品哈希绑定的 SBOM 和第三方许可清单
- **AND** 依赖来源和 Apache-2.0 通知可复核

#### Scenario: Windows 平台签名

- **WHEN** 验证 Windows 制品身份
- **THEN** 证据必须分别记录构建 provenance、Authenticode signer、签名校验和 SmartScreen/策略结果
- **AND** 测试证书不得冒充 Yonder 生产签名主体

#### Scenario: macOS 平台签名与公证

- **WHEN** 验证 macOS 制品身份
- **THEN** 证据必须分别记录构建 provenance、Developer ID signer、hardened runtime、notarization、stapling 与 Gatekeeper 结果
- **AND** ad-hoc 签名不得冒充 Yonder 生产签名主体
