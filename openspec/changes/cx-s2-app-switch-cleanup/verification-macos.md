# macOS Verification Goal：应用切换清场

状态：实施者PASS，等待独立复核

此前透明窗口失焦并未在真实Finder切换时可靠触发，旧证据不作为通过依据。现由独立的系统应用切换触发原生工作区通知，正式bundle已重新验证选择中、确认卡中切换及截图主动隐藏回归。结构化证据位于`apps/desktop/evidence/cx-s2-app-switch-cleanup-macos-20260921/result.json`，只含布尔结果，不记录应用名、截图、问题或语音正文。独立复核通过后才能Archive。Windows按用户决定继续暂缓。
