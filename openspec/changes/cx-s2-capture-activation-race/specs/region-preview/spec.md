# region-preview Delta Specification

## ADDED Requirements

### Requirement: 应用切换必须结束可见圈选会话

系统 MUST 在 macOS 可见圈选层或确认卡失去焦点时结束本轮圈选会话，但 MUST NOT 把截图主动隐藏覆盖窗口造成的预期激活变化当作用户切换应用。

#### Scenario: 截图主动隐藏不清场

- **GIVEN** 当前 `PreviewSession` 已进入 `Capturing`
- **WHEN** Yonder 主动隐藏圈选覆盖窗口且原来源应用重新成为前台应用
- **THEN** 应用切换回调不得清除当前会话
- **AND** 捕获成功后显示持有本轮内存截图的确认卡

#### Scenario: 可见阶段真实切换仍清场

- **GIVEN** 当前 `PreviewSession` 处于 `Selecting` 或 `Reviewing`
- **WHEN** 用户切换到其他应用
- **THEN** 系统取消本轮语音、清除选区与截图并隐藏圈选窗口

### Requirement: 显式圈选触发系统捕获授权

系统 MUST 只在用户完成有效圈选且当前应用身份未获屏幕录制授权时，通过 macOS 公开 API 发起系统授权请求，并 MUST 区分权限拒绝与授权后的捕获失败。

#### Scenario: 临时签名身份尚未授权

- **WHEN** 用户完成有效圈选且预检未授权
- **THEN** 系统发起一次 macOS 屏幕录制授权请求
- **AND** 用户拒绝或授权仍未生效时进入仅文字提问路径，不清空问题或创建任务

#### Scenario: 已授权但捕获失败

- **WHEN** 权限预检通过但 ScreenCaptureKit 未返回图片
- **THEN** 系统显示截图不可用的可重试反馈
- **AND** 不得误报为权限缺失
