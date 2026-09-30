# Computer Visual Fallback Delta

## ADDED Requirements

### Requirement: 元素优先的同窗口视觉降级

系统 MUST 优先使用已绑定精确窗口的AX元素；只有元素缺失、多义、不可操作或动作未确认时，才采集同一窗口截图供归属Agent重规划。

#### Scenario: 自绘应用返回空AX树

- **WHEN** 动作后同一可信窗口的AX元素数量为零且尚无视觉Observation
- **THEN** Worker只补采一次该PID/window截图，不重放动作、不采当前前台或全桌面

#### Scenario: Driver无法确认元素动作

- **WHEN** Driver返回`refused`、`partial`、`unverifiable`或`suspected-noop`
- **THEN** 后置Observe包含同一窗口临时截图，原动作结论保持失败或unknown

### Requirement: 视觉事实穿透事件驱动运行时

系统 MUST 把`UnknownObserved`已经取得的有界Observation返回Gateway，不得因异步持久化或unknown结论丢弃。

#### Scenario: 动作不可核实但截图有效

- **WHEN** Adapter返回`UnknownObserved(observe-failed)`及合法窗口截图
- **THEN** 任务保持unknown/交回语义，Gateway响应仍包含该视觉Observation

### Requirement: 通用坐标文本受可信窗口约束

系统 MUST 为通用`computer.step`窗口坐标文本注入精确target、受监管session与foreground；Agent不得覆盖这些身份字段。

#### Scenario: 慢脑根据视觉证据提交搜索框坐标

- **WHEN** Agent提交有限窗口局部`x/y`和普通查询文本
- **THEN** Worker只向当前任务已绑定窗口执行一次foreground原子文本动作并完成同窗口Observe

### Requirement: 失败步骤不显示完成

系统 MUST 仅把`action_succeeded=true`的已Observe步骤投影为完成。

#### Scenario: 动作已Observe但被拒绝

- **WHEN** 步骤到达安全边界且`action_succeeded=false`
- **THEN** 顶部执行状态显示未核实/失败，不显示成功图标

### Requirement: 完整慢脑计划投影

系统 MUST 接受通用桌面任务的封闭多步骤计划片段，并在首个动作派发前将全部槽位投影为计划总数及当前附近的有界步骤列表。

#### Scenario: 慢脑提交搜索与播放片段

- **WHEN** 协议1.40归属Agent一次提交聚焦控件、输入文本和激活控件等多个槽位
- **THEN** 顶部浮窗显示计划总数、当前附近步骤及当前/完成/待执行状态，不显示“慢脑已提交单步执行”替代计划片段
