# Verification Goal：E0-S5

复核 Windows 原生窗口探针、Chrome/Edge Native Messaging 实连、协议边界、隐私窗口排除及 Recording 默认关闭/启停证据。若读取 History DB、允许通配扩展来源、停止 Recording 后仍保留输入钩子，或缺少真实 Chrome/Edge 任一实连，Goal 失败。

当前状态：Windows 范围通过。Chrome/Edge 均完成实连；隐私窗口由浏览器未授权与扩展发送前检查双重排除；浏览器停止后无事件，Windows 低级 Hook 在 `finally` 中解除且进程退出。macOS 按已确认范围延期，不得由本 Goal 推断其可行性。
