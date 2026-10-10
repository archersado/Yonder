# AG-S10 架构设计

## 边界与依赖

不改变 Gateway、协议或持久化边界。修复限于 Application 的 `PreviewSession`、Desktop 组合根、既有 `AgentInputHub` 和圈选 WebView。

## 状态与契约

`PreviewSession` 仍是截图临时数据的原生所有者。提交状态从 `Reviewing` 进入 `Submitting`：

- `accepted`：调用 `clear`，停止圈选监听；回复展示由 AG-S9 的临时关联继续持有。
- `rejected` 或本地错误：调用 `submission_failed` 返回 `Reviewing`，图片与来源保持不变。
- `unknown`：同样保留现场，但 UI 禁止再次发送；只有用户关闭或重新圈选才能清除，不能自动重投。

WebView 只保存用户正在编辑的问题和预览投影，不成为连接或投递事实所有者。错误处理不得清空 DOM 中的问题或预览。

## 附件上限

`AgentInputHub` 在 base64 解码后、建立回复关联及调用 `submit_selection` 前，按协议唯一来源 `MAX_AGENT_ATTACHMENT_BYTES` 校验。超限返回稳定本地分类，由 Desktop 映射为 `attachment-too-large`。不静默压缩、切片改变图片或落盘。

## 失败与验证

- Application 单元测试验证失败恢复和显式清理。
- Desktop Hub 测试验证 4 MiB 边界及超限不投递。
- UI 合约检查验证失败不删除预览/问题，unknown 禁止重试。
- macOS 正式 bundle 验证提交失败保留现场及 accepted 正常链路。
