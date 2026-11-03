# Region Preview Delta

## ADDED Requirements

### Requirement: Windows受限原生截图

系统 MUST 只在用户显式圈选后，通过Windows Graphics Capture捕获96 DPI目标显示器的选区；像素 MUST 只在有界内存存在。

#### Scenario: 100%缩放单显示器选区

- **WHEN** Windows用户在96 DPI显示器内完成有效选区
- **THEN** 系统隐藏圈选层后返回与物理选区位置、尺寸一致的PNG预览
- **AND** 不保存截图、正文或像素摘要

#### Scenario: 负坐标显示器

- **WHEN** 选区位于负坐标的96 DPI显示器
- **THEN** 系统以完整显示器物理原点裁剪，不把工作区或主屏原点用于换算

### Requirement: 未验证范围必须安全降级

系统 MUST 在非96 DPI、跨显示器、超时或原生捕获失败时进入无附件确认状态，不得显示旧图、错位图或伪造成功。

#### Scenario: 非100%缩放

- **WHEN** 目标Windows显示器有效DPI不是96
- **THEN** 系统不尝试返回截图并显示本次不包含截图
- **AND** 用户仍可输入文字/语音、重新圈选或取消

#### Scenario: 捕获失败

- **WHEN** WGC不支持、超时、受保护内容或设备错误导致捕获失败
- **THEN** PreviewSession从`capturing`进入无附件`reviewing`
- **AND** 文字提交、重新圈选和取消保持合法且系统不自动重试

#### Scenario: 显示器拓扑变化

- **WHEN** Windows圈选层或确认卡存在期间显示器位置、尺寸或缩放发生变化
- **THEN** 系统取消本轮语音、释放截图和会话并隐藏窗口
- **AND** 观察线程随会话清场退出，不保留跨会话监听
