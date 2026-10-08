# protocol-single-source Delta Specification

## ADDED Requirements

### Requirement: 协议版本常量单一来源

当前发布协议版本 MUST 仅定义于 `yonder-protocol`；组合根与 CLI 的协议握手 MUST 引用该常量，MUST NOT 在 `apps/` 下书写 minor 版本字面量，除非该处声明了有意钉定的能力边界及理由。

#### Scenario: 协议升版后 CLI 自动跟随

- **WHEN** `yonder-protocol` 的 `PROTOCOL_VERSION` minor 递增且 CLI 重新编译
- **THEN** CLI hello 握手携带新版本，不因字面量遗漏而静默停在旧版本

#### Scenario: 有意钉定须显式声明

- **WHEN** 某调用方（如 Codex 桥）只依赖既有能力面而钉定旧 minor
- **THEN** 该处 MUST 以注释声明理由，且架构检查可区分"引用常量"与"钉定例外"

### Requirement: 内部错误分类到 wire 枚举的唯一映射

执行未知原因（UnknownReason）的内部枚举与 wire 枚举之间的映射 MUST 仅存在一个 `From` 实现；Gateway 与查询投影 MUST 通过该映射转换，MUST NOT 各自维护手写分支。

#### Scenario: 新增 reason 漏映射即编译失败

- **WHEN** 内部枚举新增一个 reason 值而未更新唯一 From 实现
- **THEN** 编译失败，而不是握手或查询时静默丢弃该分类

#### Scenario: 映射形状回归锁定

- **WHEN** 运行映射回归测试
- **THEN** 逐值断言内部枚举经映射后序列化为 kebab-case wire 名，形状漂移即测试失败

### Requirement: ID 规则单一实现

ID 合法性字符集规则 MUST 仅实现于 `yonder-protocol` 的 `valid_id`；其他 crate MUST 引用该实现或仅做错误类型映射，MUST NOT 复制字符集判断。

#### Scenario: 规则变更只改一处

- **WHEN** ID 字符集或长度规则调整
- **THEN** 仅 `yonder-protocol::valid_id` 一处实现变化，application/adapters/desktop 行为经引用自动一致
