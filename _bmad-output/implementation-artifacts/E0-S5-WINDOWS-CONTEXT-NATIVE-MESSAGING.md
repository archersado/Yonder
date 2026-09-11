# E0-S5 Windows 上下文与 Native Messaging 验证

## Story

作为 Yonder 开发团队，我们需要验证 Windows 前台窗口元数据采集与 Chrome/Edge Native Messaging 通道，以确定个人上下文采集能否进入产品研发。

## 验收条件

1. 使用 Windows 原生 API 获取当前前台进程、应用和窗口元数据，不轮询全盘或浏览器数据库。
2. Recording 默认关闭；只有显式开始后才允许采集用户操作，停止后立即解除钩子。
3. Native Messaging 正确处理 UTF-8、分片输入、连续消息、32 位本机字节序长度头与 1 MiB 出站上限。
4. Host 标准输出只承载协议，诊断只写标准错误。
5. Chrome 与 Edge 使用当前用户级注册、精确扩展来源；不允许通配来源。
6. 扩展不申请 History 权限，隐私窗口不采集。
7. 形成结构化 Windows 证据与 AD-E0-05；macOS 证据按当前决策延期，不得据此宣称跨平台完成。

OpenSpec：`openspec/changes/e0-validate-windows-context/`
