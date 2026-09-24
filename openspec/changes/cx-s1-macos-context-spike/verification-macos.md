# 独立Verification Goal：macOS原生上下文与Native Messaging

日期：2026-09-24  
结论：FAIL（返回Apply）

## 环境与已通过子项

- macOS 26.5.1，Apple Swift 6.3.3，Accessibility已授权。
- `check-protocol.py`通过UTF-8分片、连续帧、stdout隔离、1 MiB超限拒绝和非法JSON拒绝。
- `check-extension.py`确认扩展不申请History权限，存在`tab.incognito`发送前拒绝，隔离测试调用`isAllowedIncognitoAccess()`，Native Host只允许固定扩展来源。
- 用户在真实Google Chrome 153.0.8010.53当前配置中显式加载固定ID扩展；当前用户级Host收到`recording.started`、`tab.activated`、`recording.stopped`三条消息并逐条回执，`isAllowedIncognitoAccess()`确认隐私模式未授权。浏览器子范围通过，独立证据见[chrome-native-messaging.json](../../../spikes/macos-context-native-messaging/evidence/macos-20260924/chrome-native-messaging.json)。
- 原生探针在一次探索运行中观察到2次应用激活、3次Observer注册和2次AX事件，停止后资源释放；但后续统一脚本重复运行得到0次激活和0次AX事件，不满足可复现门槛，因此原生事件路线不得记为通过。
- 结构化证据：[result.json](../../../spikes/macos-context-native-messaging/evidence/macos-20260924/result.json)。证据不含窗口标题、URL、页面标题、键值、坐标或Payload。

## 浏览器验证范围

官方Chrome从137开始移除稳定版`--load-extension`开关，因此自动隔离样本不能替代真实稳定版验证。本轮由用户通过Chrome系统目录选择器显式加载扩展，再打开固定扩展自检页；Host实际收到三条有界消息，扩展来源精确匹配，且隐私模式访问被拒绝。Google Chrome子范围通过。

Microsoft Edge未安装，未产生Edge Native Messaging真实回执；Chrome通过不代表Edge通过。

## 结论

Google Chrome Native Messaging门槛已完成；macOS原生事件投递仍不可重复，因此Goal继续失败并返回Apply。AD-CX-03保持Proposed，CX-S1保持`design-review`，不得接产品Context Port。下一轮须用签名/Bundle化测试宿主复核RunLoop与TCC边界；Edge待目标环境安装后单独验证。

官方依据：[Apple NSWorkspace激活通知](https://developer.apple.com/documentation/appkit/nsworkspace/didactivateapplicationnotification)、[Apple AXObserver](https://developer.apple.com/documentation/applicationservices/1460133-axobservercreate)、[Chrome Native Messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging)、[Chrome 137移除load-extension](https://developer.chrome.com/blog/extension-news-june-2025)、[Chrome for Testing](https://developer.chrome.com/docs/chrome-for-testing/)。
