# macOS 安全输入门禁 Verification Goal

结论：子范围 PASS。

2026-09-20 在 macOS「系统设置 → 触控 ID 与密码 → 更改密码」的系统认证框聚焦期间，探针报告 `secure_input_active=true`，没有采集事件或任何正文、坐标、截图、AX 文本。随后已取消认证窗口，没有修改系统密码。

结构化证据：[result.json](../../../spikes/recording-capture/evidence/secure-input-macos-20260920/result.json)。
