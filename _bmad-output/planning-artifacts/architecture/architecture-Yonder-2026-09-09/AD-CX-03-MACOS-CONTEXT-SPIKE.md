# AD-CX-03 macOS原生上下文技术路线

- 状态：Accepted（macOS原生事件与Google Chrome；Microsoft Edge未验证）
- Story：CX-S1
- OpenSpec：`cx-s1-macos-context-spike`
- 日期：2026-09-24
- 期限：2026-09-26

## 背景

架构主干要求macOS当前窗口使用`NSWorkspace/AXObserver`事件驱动采集，日常浏览使用Chrome/Edge扩展经Native Messaging上报。Accepted AD-E0-05只接受Windows路线并明确不构成macOS证据；2026-09-24用户要求在暂缓Windows验证的前提下继续推进，因此先完成macOS统一Spike，不接产品运行时。

## 候选决定

- 使用`NSWorkspace.didActivateApplicationNotification`观察应用切换；只在Accessibility已授权时，使用绑定当前PID的`AXObserver`观察焦点窗口变化。
- AX服务可能晚于应用激活通知就绪；首次Observer注册失败时，仅对仍处于前台的同一PID执行一次500ms有界重试。RunLoop Source使用common modes，停止时显式移除。
- 使用Manifest V3扩展与macOS当前用户级Native Messaging Host manifest建立浏览通道；Host遵循32位本机字节序长度头、1 MiB帧上限、精确扩展来源和stdout协议隔离。
- Spike只输出无正文结构化证据，不创建Context Port实现、SQLite表、Recording状态、索引或同步连接。

## 统一样本与淘汰门槛

统一样本验证应用激活、窗口Observer权限与释放、Native Messaging UTF-8/分片/连续消息/超限拒绝、Chrome真实连接、隐私窗口拒绝和停止后无残留Host。Accessibility未授权、浏览器未安装或扩展未连接必须明确失败，不允许轮询、截图、History数据库或通配来源兜底。

若系统事件不能稳定到达、AXObserver授权后无法可靠清理、Host不能限制帧或隔离stdout、扩展必须申请History/隐私权限，路线淘汰。Google Chrome与Microsoft Edge分别验证，单一浏览器通过不得代表另一浏览器。

2026-09-24证据结论：两个ad-hoc签名测试应用统一样本连续三轮通过，每轮4次激活、5次Observer有效注册和8至9次AX窗口事件，资源均释放；Google Chrome 153.0.8010.53当前用户级Native Messaging三条消息实连通过，隐私模式权限被拒绝。Microsoft Edge未安装，保持未验证且不得由Chrome证据替代。

## 架构影响

Architecture Impact：none（Spike）。不改变模块依赖、协议、持久化、系统边界或状态所有者。通过后仍需单独的产品Story、Accepted ADR和OpenSpec才能接入Desktop与Context Port。
