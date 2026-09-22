# 接管工作定位 Delta

## ADDED Requirements

### Requirement: 可信引用
系统 MUST 只从当前CUA步骤的可信目标和有效Observe捕获WorkRef；引用 MUST 绑定task/step/attempt/Worker/host且只在宿主内存保存。

#### Scenario: 拒绝脱离当前步骤的引用

- **WHEN** 目标不来自当前CUA步骤的可信目标或缺少有效Observe
- **THEN** 不捕获WorkRef，也不进入定位

### Requirement: 停止后定位
系统 MUST 在takeover停止事实提交后释放任务执行占用、保持桌面接管闸并提交locating，随后才调用WorkFocusPort；暂停和取消 MUST NOT 定位。

#### Scenario: 仅接管进入locating

- **WHEN** takeover停止事实提交成功
- **THEN** 任务执行占用释放、桌面接管闸保持，任务提交locating后才调用WorkFocusPort

### Requirement: 定位事实原子发布
locating与最终focused/failed MUST 分别和任务sequence、事件、Outbox同事务；UI MUST 只显示已提交阶段，不以原生调用返回替代事实。

#### Scenario: UI只显示已提交阶段

- **WHEN** WorkFocusPort返回但定位状态尚未提交
- **THEN** 界面不显示focused或failed

### Requirement: 保守失败
引用失效、身份变化、不唯一、权限、激活或新鲜核验失败 MUST 保持paused并返回稳定失败；系统 MUST NOT 改选同名窗口、移动窗口、自动重试或启动Recording。

#### Scenario: 引用失效保持paused

- **WHEN** WorkRef身份变化或新鲜核验失败
- **THEN** 任务保持paused并返回稳定失败，不改选窗口或启动Recording
