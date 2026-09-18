# Work Focus Delta

## Requirements

### Requirement: 可信WorkRef
系统 MUST 只从当前可信attempt与有效Observe的WorkTarget捕获WorkRef，并绑定task/step/attempt/Worker/host及进程启动身份。

### Requirement: 定位前复核
系统 MUST 在任何原生前置副作用前复核权限、进程启动身份、WindowServer与保留AX对象的双侧唯一映射；失败 MUST 拒绝且不得改选同名目标。

### Requirement: 新鲜成功证明
系统 MUST 在恢复最小化和前置后重新读取Frontmost、FocusedWindow、最小化及几何；只有全部匹配才返回成功，且不得移动窗口或启动Recording。
