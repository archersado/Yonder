# 独立Verification Goal：macOS原生上下文与Native Messaging

日期：2026-09-24  
结论：FAIL（返回Apply）

## 环境与已通过子项

- macOS 26.5.1，Apple Swift 6.3.3，Accessibility已授权。
- `check-protocol.py`通过UTF-8分片、连续帧、stdout隔离、1 MiB超限拒绝和非法JSON拒绝。
- `check-extension.py`确认扩展不申请History权限，存在`tab.incognito`发送前拒绝，隔离测试调用`isAllowedIncognitoAccess()`，Native Host只允许固定扩展来源。
- 原生探针在一次探索运行中观察到2次应用激活、3次Observer注册和2次AX事件，停止后资源释放；但后续统一脚本重复运行得到0次激活和0次AX事件，不满足可复现门槛，因此原生事件路线不得记为通过。
- 结构化证据：[result.json](../../../spikes/macos-context-native-messaging/evidence/macos-20260924/result.json)。证据不含窗口标题、URL、页面标题、键值、坐标或Payload。

## 浏览器阻断

本机Google Chrome 153.0.8010.53可用，但官方Chrome从137开始移除稳定版`--load-extension`开关。Ego Lite扩展页可以进入开发者模式，加载未打包扩展仍需要浏览器所有的系统目录选择器；自动化不能代替用户确认。匹配版本Chrome for Testing临时下载因环境吞吐过低中止并清理，未将其记为通过。

Microsoft Edge未安装。当前没有Google Chrome/Edge Native Messaging真实回执。

## 结论

两个核心门槛均未完成：原生事件投递不可重复，浏览器实连未确认。Goal失败返回Apply；AD-CX-03保持Proposed，CX-S1保持`design-review`，不得接产品Context Port。下一轮须先用签名/Bundle化测试宿主复核RunLoop与TCC边界，再由用户显式加载Chrome扩展或使用可用的Chrome for Testing固定版本。

官方依据：[Apple NSWorkspace激活通知](https://developer.apple.com/documentation/appkit/nsworkspace/didactivateapplicationnotification)、[Apple AXObserver](https://developer.apple.com/documentation/applicationservices/1460133-axobservercreate)、[Chrome Native Messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging)、[Chrome 137移除load-extension](https://developer.chrome.com/blog/extension-news-june-2025)、[Chrome for Testing](https://developer.chrome.com/docs/chrome-for-testing/)。
