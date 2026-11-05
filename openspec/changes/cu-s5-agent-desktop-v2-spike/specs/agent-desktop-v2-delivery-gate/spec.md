# agent-desktop v2 交付门禁 Delta Spec

## Requirement: v2 构建来源隔离

系统 MUST 固定与 CU-S4 相同的上游 revision，并为 v2 使用独立 patch set、源码目录、manifest 与样本版本；MUST NOT 修改或借用 v1 的来源证明。

### Scenario: v1 来源未变

- **GIVEN** v1 build manifest 与补丁哈希已冻结
- **WHEN** 准备 v2 源码
- **THEN** v1 目录与 manifest 哈希不变，v2 证据引用自己的 patch/source/sample 哈希

## Requirement: 最终交付对象连续

Worker MUST 在派发时使用 snapshot 所保留的原生元素对象；任何 stale、替代对象重绑定、重新解析或无法证明的物理交付 MUST 拒绝且不得重试。

### Scenario: 同窗元素替换

- **GIVEN** snapshot 保留按钮 A，fixture 销毁 A 并在同窗创建同属性按钮 B
- **WHEN** v2 只读门禁检查旧引用及原生对象
- **THEN** 旧对象不可被替换为 B；原生动作保持拒绝，两个按钮副作用计数均为零

### Scenario: 整窗重建或窗口 ID 复用

- **GIVEN** 原窗口已销毁，新窗口具有相同名称、元素属性或原生窗口 ID
- **WHEN** 验证交付路径
- **THEN** 必须证明原对象失效且路径不会绑定新对象；无法制造复用样本记未验证，不计通过

## Requirement: 输入归属与失效冻结

Host MUST 区分当前 attempt/Worker、用户与未知输入；observer 的持续有效性无法证明时 MUST 冻结派发。

### Scenario: 其他进程注入相同标记

- **GIVEN** 其他进程生成与当前标记相同的输入元数据
- **WHEN** Host 分类事件
- **THEN** 不得仅凭元数据将其归为当前 Worker；归属不明时递增 generation 并冻结

### Scenario: 观察器静默失效

- **GIVEN** Windows 低级钩子线程仍存活但钩子可能已被系统移除
- **WHEN** Supervisor 判断是否准入动作
- **THEN** `observer_ready=false`，动作 `unsupported/not_delivered`

## Requirement: 分阶段准入

系统 MUST 在 Windows/macOS 两项机制门禁与独立 Verification Goal 通过并修订 ADR 前保持原生 dispatch 关闭；缺环境记 `environment_unavailable`，不得以单平台证据替代。
