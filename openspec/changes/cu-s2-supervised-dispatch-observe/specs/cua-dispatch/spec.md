# CUA 受监管派发

## ADDED Requirements

### Requirement: 完整身份派发
系统必须只接受完整 task/step/attempt/Worker/host 身份一致的受监管 Worker 结果；身份缺失或不一致不得改变任务事实。

### Requirement: 动作后强制 Observe
每次后台 AX 输入返回后必须执行后置 Observe；只有动作结论已知且 Observe 有效才能返回已知结果，不能用 SDK 调用返回代替观察。

### Requirement: 未知副作用保守处理
Worker 超时、崩溃、断连、非法回包或 Observe 失败必须返回 unknown，不得自动重试动作或释放执行占用。

### Requirement: SDK-only Worker
Worker 只加载固定 trycua SDK 和匹配原生库，通过继承 stdio 受 Yonder 监管；不得启动上游 App、开放本地 HTTP/TCP 或建立第二状态源。
