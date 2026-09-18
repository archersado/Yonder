# CX-S1 架构设计

## 边界与依赖

依据 AD-E0-05，原生窗口事件与 Native Messaging，队列有界；数据经 Context Port。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

Windows 已验证，macOS 延期；产品采集与检索接线待定。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。
