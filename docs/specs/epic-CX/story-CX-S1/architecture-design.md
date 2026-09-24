# CX-S1 架构设计

## 边界与依赖

Windows依据Accepted AD-E0-05使用原生窗口事件与Native Messaging。macOS新增Proposed AD-CX-03：应用切换使用`NSWorkspace.didActivateApplicationNotification`，窗口变化候选使用绑定当前进程的`AXObserver`；浏览器使用Manifest V3扩展和当前用户目录下的Native Messaging Host manifest。Spike保持独立目录和有界进程内计数，不进入Context Port。

本Spike不改变Workspace依赖、Rust协议、SQLite schema或状态所有者，Architecture Impact为`none`。若后续产品化，Desktop组合根只能把原生事件交给Application定义的Context Port；Native Host只能校验并转发，不能直接持久化或调用其他Adapter。

## 状态与契约

MVP 按 Accepted AD-ST-01 使用未加密 SQLite 保持任务当前事实源，SQLCipher 延期至 ST-S2；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

Spike生命周期由测试进程持有：启动后注册`NSWorkspace`通知；仅在Accessibility已授权时为当前前台进程注册`AXObserver`；应用切换时先移除旧Observer再绑定新进程；停止、超时和异常都移除通知与RunLoop source。只输出固定结果码、Bundle ID是否存在、PID有效性、窗口属性是否可访问、事件计数和资源释放状态。

Chrome Native Host使用32位本机字节序长度头，单帧上限1 MiB，完整读取后才解析JSON；stdout只写协议响应，诊断写stderr且不含Payload。用户级manifest必须使用绝对Host路径和精确`chrome-extension://<id>/`来源。扩展继续不申请`history`权限，并在`tab.incognito`时拒绝发送。

## 失败与验证

Windows已验证；macOS Spike覆盖正常、权限拒绝、超限帧、分片/连续帧、隐私拒绝和停止释放。失败不得隐式重试或降级为轮询、截图、History读取；Google Chrome与Microsoft Edge证据分别声明，缺少浏览器不能记为通过。

统一淘汰门槛：无法通过系统事件稳定得到应用切换、AXObserver不能在授权后释放、Host无法限制帧或隔离stdout、扩展必须申请History/隐私访问，任一成立即拒绝路线并保持产品能力disabled。Spike期限至2026-09-26，届时必须形成Verification Goal与ADR结论或记录失败。

## 架构影响

本次只授权`spikes/macos-context-native-messaging/`和验证文档；不得修改Desktop运行时或生成产品协议。产品接线仍须新的Architecture Decision与OpenSpec。
