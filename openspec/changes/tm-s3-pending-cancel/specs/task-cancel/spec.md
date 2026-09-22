# 未开始任务取消

## ADDED Requirements

### Requirement: 授权取消
Agent MUST 仅所属、LocalUser任意created可取消；归属/版本/过期/序号不符 MUST 拒绝，无额外写。Gateway需要1.2。本机控制绑定固定用户身份。

#### Scenario: 非归属或过期请求被拒绝

- **WHEN** Agent提交不归属任务的取消，或归属、版本、过期时间、序号任一不符
- **THEN** 请求被拒绝，任务和事件不变

### Requirement: 原子与不复活
Cancel MUST 复用三表CAS事务；重复同取消前序号或当前序号 MUST 返回当前快照，不增事件；其他序号 MUST 冲突。非created MUST 拒绝，不把请求当停止，创建重试 MUST NOT 复活。

#### Scenario: 重复取消幂等

- **WHEN** 相同取消在已取消任务上按前序号或当前序号重投
- **THEN** 返回当前快照，不追加事件

### Requirement: 详情交互
created详情 MUST 提供取消按钮，提交中禁用，成功刷新列表，全部保留终态；失败 MUST 提示，不自动重试。原生验证只影响目标任务，Windows门禁保留。

#### Scenario: 详情失败反馈

- **WHEN** 取消请求失败或结果未知
- **THEN** 详情提示失败，按钮不再提交，任务不由界面自动重试取消
