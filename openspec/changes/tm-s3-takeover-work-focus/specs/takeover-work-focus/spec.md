# 接管工作定位 Delta

## ADDED Requirements

### Requirement: 可信引用
系统 MUST 只从当前CUA步骤的可信目标和有效Observe捕获WorkRef；引用 MUST 绑定task/step/attempt/Worker/host且只在宿主内存保存。

### Requirement: 停止后定位
系统 MUST 在takeover停止事实提交后释放任务执行占用、保持桌面接管闸并提交locating，随后才调用WorkFocusPort；暂停和取消 MUST NOT 定位。

### Requirement: 定位事实原子发布
locating与最终focused/failed MUST 分别和任务sequence、事件、Outbox同事务；UI MUST 只显示已提交阶段，不以原生调用返回替代事实。

### Requirement: 保守失败
引用失效、身份变化、不唯一、权限、激活或新鲜核验失败 MUST 保持paused并返回稳定失败；系统 MUST NOT 改选同名窗口、移动窗口、自动重试或启动Recording。
