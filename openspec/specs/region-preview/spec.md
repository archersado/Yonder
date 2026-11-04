# region-preview Specification

## Purpose
定义 Yonder 在 macOS 单显示器上的显式短生命圈选预览：区域选择、仅内存截图、确认卡、CUA租约协调及所有结束路径清场。

## Requirements

### Requirement: 显式且短生命的区域选择

系统 MUST 只在用户明确启动且桌面租约协调完成后，于当前显示器提供一次区域选择，并在结束时清理临时截图。

#### Scenario: 框选或笔画成功选择

- Given macOS屏幕捕获权限可用且桌面租约协调完成
- When 用户选择框选后拖出有效区域，或选择笔画后画出有效笔画并松开
- Then 笔画以其外接矩形作为截图范围
- Then 系统显示本地确认卡，截图只在内存保留且不会发送或持久化

#### Scenario: 取消或拒绝

- Given 用户按Esc、取消、超时、权限缺失或桌面任务停止结果不能确认
- When 圈选会话结束
- Then 系统清理选择层、选区、笔画和截图字节，不创建任务、不发送Agent输入
- Then 再次打开时不得显示上一会话的预览

### Requirement: 圈选前暂停桌面任务

系统 MUST 在CUA持有桌面租约时先以可信本地用户身份暂停租约所属任务，并只在步骤边界停止事实提交后显示圈选层。

#### Scenario: 已观察步骤安全暂停

- **WHEN** 用户从小龙或托盘启动圈选，当前桌面任务的动作已返回且Observe有效
- **THEN** 系统提交`pause`、将任务原子转为paused并释放桌面租约
- **AND** 随后自动显示圈选层，不定位窗口、不启动Recording、不派发新CUA动作

#### Scenario: 停止结果不能确认

- **WHEN** 当前attempt为prepared、unknown，或身份、序号、存储提交失败
- **THEN** 系统保持桌面租约与任务事实，不显示圈选层
- **AND** 返回稳定的结果待核实反馈，不轮询数据库、不自动重试

#### Scenario: 当前没有桌面租约

- **WHEN** 用户启动圈选且没有任务持有桌面租约
- **THEN** 系统不创建控制记录，直接进入既有圈选流程

### Requirement: 圈选语音直接提交

系统 MUST 只在用户从当前圈选确认卡显式开始语音后采集，并将最终非空转写作为当前selection请求直接提交。

#### Scenario: 携带当前截图提交

- **WHEN** 确认卡持有临时截图，用户点击麦克风并产生最终非空转写
- **THEN** 部分转写只显示在问题框，最终转写自动复用当前附件提交
- **AND** 系统只产生一条selection输入，不另行产生voice输入或要求再次点击发送

#### Scenario: 无截图语音提交

- **WHEN** 当前确认卡因小选区或权限缺失没有截图，用户完成语音转写
- **THEN** 系统以selection来源自动提交一条无附件输入

#### Scenario: 圈选会话结束时清场

- **WHEN** 用户关闭、取消、重新圈选，或圈选会话超时
- **THEN** 系统立即取消本轮region语音采集并丢弃未完成转写
- **AND** 后续最终事件不得提交给普通voice或旧PreviewSession

#### Scenario: 稳定提交结果

- **WHEN** 最终转写提交返回accepted、rejected或unknown
- **THEN** 确认卡复用既有selection提交反馈与附件清理语义
- **AND** unknown不得自动重试

### Requirement: 来源应用仅在本轮确认卡显示

系统 MUST 在用户显式启动圈选时读取一次有界来源应用显示名称，并只在当前确认卡显示。

#### Scenario: 显示可信来源

- **WHEN** 平台在覆盖层出现前返回有效前台应用名称
- **THEN** 确认卡显示“来源：应用名”
- **AND** 重新圈选继续显示同一来源

#### Scenario: 来源不可用

- **WHEN** 平台未返回名称或名称不满足有界校验
- **THEN** 确认卡显示“来源：当前桌面”

#### Scenario: 来源数据边界

- **WHEN** 圈选会话关闭、取消、提交或超时
- **THEN** PreviewSession清除来源名称
- **AND** 来源名称不进入Agent正文、协议、日志、SQLite、事件或Outbox

### Requirement: Windows 192 DPI有界截图

系统 MUST 只在目标显示器有效DPI为同轴96/96或192/192、选区完全位于单个显示器且不超过既有限额时返回截图。CSS选区 MUST 只换算一次到物理像素，截图 MUST 只在有界内存存在。

#### Scenario: 200%主屏显示截图

- **GIVEN** Windows目标显示器为192 DPI且用户在单个显示器内完成有效圈选
- **WHEN** Desktop将CSS选区换算并调用共享WGC模块
- **THEN** 系统返回对应物理区域的PNG并在确认卡显示，不得返回`capture-unsupported-scale`

#### Scenario: 200%逻辑与物理尺寸一致

- **GIVEN** 200%显示器上240×160的CSS选区
- **WHEN** 系统按窗口viewport与物理work area换算
- **THEN** 捕获矩形为对应的480×320物理像素且内容位置一致，不得重复乘DPI

### Requirement: 未验证缩放继续失败关闭

系统 MUST 在120/144/168或异轴DPI、跨屏、超限、超时或WGC失败时清除旧图并进入无附件`reviewing`，不得自动重试。

#### Scenario: 未验证DPI降级

- **GIVEN** 目标显示器不是同轴96或192 DPI
- **WHEN** 用户完成圈选
- **THEN** 确认卡显示本次不含截图，并保留文字、语音、重选和取消

#### Scenario: 跨屏选区降级

- **GIVEN** 选区跨越两个显示器
- **WHEN** 用户完成圈选
- **THEN** 系统拒绝截图并进入无附件`reviewing`，不得自动重试

#### Scenario: 超限选区降级

- **GIVEN** 单屏选区的尺寸、像素数或编码结果超过既有限额
- **WHEN** 系统校验或编码截图
- **THEN** 系统清除旧图并进入无附件`reviewing`，不得返回超限图片

#### Scenario: 捕获超时降级

- **GIVEN** WGC未在既有超时限制内完成
- **WHEN** 捕获等待超时
- **THEN** 系统清除旧图并进入无附件`reviewing`，不得自动重试

#### Scenario: WGC失败降级

- **GIVEN** 目标显示器与选区受支持但WGC返回失败
- **WHEN** 系统处理捕获结果
- **THEN** 系统清除旧图并进入无附件`reviewing`，保留文字、语音、重选和取消

#### Scenario: 旧捕获不得污染新会话

- **GIVEN** 用户已重选、取消或开始新的Preview generation
- **WHEN** 旧异步捕获随后返回
- **THEN** 系统拒绝旧结果并释放图片字节，不得覆盖当前确认卡
