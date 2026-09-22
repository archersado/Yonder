# CUA 受监管派发

## ADDED Requirements

### Requirement: 完整身份派发
系统必须只接受完整 task/step/attempt/Worker/host 身份一致的受监管 Worker 结果；身份缺失或不一致不得改变任务事实。

#### Scenario: 身份不一致拒绝结果

- **WHEN** Worker回包缺少task、step、attempt、Worker或host任一身份
- **THEN** 拒绝该结果，任务事实不变

### Requirement: 动作后强制 Observe
每次后台 AX 输入返回后必须执行后置 Observe；只有动作结论已知且 Observe 有效才能返回已知结果，不能用 SDK 调用返回代替观察。

#### Scenario: SDK返回不替代Observe

- **WHEN** SDK动作调用返回但后置Observe失败
- **THEN** 结果为unknown，不向Agent返回已知动作结论

### Requirement: 未知副作用保守处理
Worker 超时、崩溃、断连、非法回包或 Observe 失败必须返回 unknown，不得自动重试动作或释放执行占用。

#### Scenario: Worker超时保持占用

- **WHEN** 受监管Worker超时或崩溃
- **THEN** attempt记录unknown，任务占用保持且不自动重试

### Requirement: SDK-only Worker
Worker 只加载固定 trycua SDK 和匹配原生库，通过继承 stdio 受 Yonder 监管；不得启动上游 App、开放本地 HTTP/TCP 或建立第二状态源。

#### Scenario: Worker禁止开放网络

- **WHEN** Worker启动或运行
- **THEN** 只加载固定SDK与匹配原生库，不监听本地HTTP/TCP或建立第二任务状态源
