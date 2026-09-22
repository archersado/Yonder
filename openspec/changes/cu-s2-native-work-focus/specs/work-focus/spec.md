# Work Focus Delta

## ADDED Requirements

### Requirement: 可信WorkRef
系统 MUST 只从当前可信attempt与有效Observe的WorkTarget捕获WorkRef，并绑定task/step/attempt/Worker/host及进程启动身份。

#### Scenario: 拒绝脱离当前attempt的目标

- **WHEN** WorkTarget不是当前可信attempt的有效Observe结果
- **THEN** 不捕获WorkRef，也不产生定位副作用

### Requirement: 定位前复核
系统 MUST 在任何原生前置副作用前复核权限、进程启动身份、WindowServer与保留AX对象的双侧唯一映射；失败 MUST 拒绝且不得改选同名目标。

#### Scenario: 双侧映射不唯一拒绝定位

- **WHEN** WindowServer与保留AX对象不是唯一双向匹配
- **THEN** 定位拒绝，不选择同名窗口或产生前置副作用

### Requirement: 新鲜成功证明
系统 MUST 在恢复最小化和前置后重新读取Frontmost、FocusedWindow、最小化及几何；只有全部匹配才返回成功，且不得移动窗口或启动Recording。

#### Scenario: 前置后状态不匹配

- **WHEN** 恢复最小化和前置后Frontmost、FocusedWindow、最小化或几何任一不匹配
- **THEN** 返回定位失败，不移动窗口或启动Recording
