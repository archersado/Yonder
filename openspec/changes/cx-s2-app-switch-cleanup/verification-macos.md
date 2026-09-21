# macOS Verification Goal：应用切换清场

状态：实施者PASS，等待独立复核

正式`com.yonder.desktop` bundle已在真实Finder切换环境验证：选择中切换应用清场、确认卡中切换应用清场，以及截图主动隐藏后仍可进入确认卡。结构化证据位于`apps/desktop/evidence/cx-s2-app-switch-cleanup-macos-20260921/result.json`，只含布尔结果，不记录应用名、截图、问题或语音正文。

`cargo test --workspace --locked`共61项通过；严格OpenSpec、架构关联、JavaScript与Swift语法检查及`git diff --check`通过。独立复核必须从当前提交重建正式bundle并复跑同一三条原生路径，确认圈选入口成功后不残留“正在暂停当前任务…”临时文案；通过后才能Archive。Windows按用户决定继续暂缓。
