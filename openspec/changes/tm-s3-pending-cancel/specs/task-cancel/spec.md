# 未开始任务取消

## ADDED Requirements

### Requirement: 授权取消
Agent仅所属、LocalUser任意created可取消；归属/版本/过期/序号不符拒绝，无额外写。Gateway需要1.2。本机控制绑定固定用户身份。

### Requirement: 原子与不复活
Cancel复用三表CAS事务；重复同取消前序号或当前序号返回当前快照，不增事件；其他序号冲突。非created拒绝，不把请求当停止，创建重试不复活。

### Requirement: 详情交互
created详情提供取消按钮，提交中禁用，成功刷新列表，全部保留终态；失败提示，不自动重试。原生验证只影响目标任务，Windows门禁保留。
