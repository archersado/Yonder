# Region Preview Delta Spec

## ADDED Requirements

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
