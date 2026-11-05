# 平台、安全与接管研究摘要（Round 1）

研究者结论：不建议原样作为强安全边界CUA Driver；可作为accessibility-first执行引擎候选，但必须由上层补齐用户接管、全局租约、取消、强制Observe、确认UI和签名。

## Findings

1. `{claim: "主线支持macOS 13+的AX、Windows 10 1809+/Server 2019+的UIA、Ubuntu 24.04+的AT-SPI；npm含三平台x64/ARM64原生二进制与helper。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md ; https://github.com/vercel-labs/agent-desktop/blob/main/docs/install.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "平台支持"}`
2. `{claim: "macOS宿主需Accessibility，截图另需Screen Recording；Windows同完整性无需提示，高完整性输入要求调用方同样提权；安全桌面、UAC、登录/锁屏、Session 0和其他用户会话不支持。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop-macos/SKILL.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop-windows/SKILL.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "原生安全边界"}`
3. `{claim: "窗口列表提供window id/title/app/pid/bounds/focus，歧义返回AMBIGUOUS_TARGET；Windows/Linux ref按捕获规则且动作前live read唯一匹配，macOS优先元素自身AX address，身份变化返回STALE_REF。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/SKILL.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-system.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop-macos/SKILL.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "目标绑定"}`
4. `{claim: "动作前校验visible/stable/enabled/supported_action/policy/editable，指针还查receives_events；但hit-test不确定时receives_events会fail open，raw --xy无窗口身份，因此不能泛化为强焦点漂移保护。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-interaction.md ; https://github.com/vercel-labs/agent-desktop/blob/main/docs/json-output.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "焦点风险"}`
5. `{claim: "Windows press写当前foreground queue，无per-process targeting；type使用SendInput并需headed，缺可读selection时可能delivered_unverified，PID/HWND绑定不覆盖所有物理输入。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop-windows/SKILL.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "目标绑定缺口"}`
6. `{claim: "interaction lease/hold_cursor是进程内动作协调；项目支持多个agent/session，PR演示两个session操作Calculator。因此它不是跨进程全桌面唯一租约。", source: "https://github.com/vercel-labs/agent-desktop/pull/347 ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-system.md", publisher: "vercel-labs/agent-desktop", pub_date: "PR 2026-10-31", accessed: "2026-11-04", confidence: "高", class: "桌面租约"}`
7. `{claim: "未发现全局用户键鼠监听、synthetic/user区分和用户输入立即暂停；held-input命令因无daemon保证释放而fail closed；未发现执行中cancel，session end不是取消协议。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-system.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-interaction.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "中高", class: "接管缺口"}`
8. `{claim: "Observe→Act→Observe由调用agent负责；仅部分stateful action自动回读。delivery disposition可映射unknown，但项目不会持久化任务unknown状态。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/SKILL.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop/references/commands-interaction.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "Observe与unknown"}`
9. `{claim: "项目是CLI/C library/tool，无GUI、模型或交互确认UI；cursor overlay只是可见光标，危险快捷键依赖--force拒绝，不是用户确认门禁。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md ; Windows/macOS SKILL.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "控制提示与同意"}`
10. `{claim: "Windows官方release未签名，可能触发SmartScreen/防病毒；Smart App Control会阻止，需关闭或自行签名。SHA-256与GitHub attestation不等于Authenticode。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/docs/install.md ; https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/skills/agent-desktop-windows/references/troubleshooting.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "代码签名"}`
11. `{claim: "snapshot/clipboard/screenshot/notification/command output/debug artifact可能含敏感内容；session默认trace并写~/.agent-desktop。结构化trace会脱敏部分字段，但显式screenshots PNG/refmap未脱敏。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md ; commands-system.md ; commands-interaction.md", publisher: "vercel-labs/agent-desktop", pub_date: "main持续更新", accessed: "2026-11-04", confidence: "高", class: "隐私"}`
12. `{claim: "SECURITY.md仅提供私密报告渠道，未声明威胁模型、支持期或SLA；v1.1.0于2026-11-03发布，集成需固定tag/commit。", source: "https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/SECURITY.md ; https://github.com/vercel-labs/agent-desktop/releases", publisher: "vercel-labs/agent-desktop", pub_date: "v1.1.0 2026-11-03", accessed: "2026-11-04", confidence: "高", class: "项目保证"}`

## Contradictions resolved

- ref-headed路径有部分焦点保护，但hit-test fail-open、raw坐标和Windows全局按键路径不能满足强窗口绑定。
- 文档倡导每动作Observe，但实现并非Driver强制不变量。
- interaction lease是局部协调，不是全局单CUA租约。
- build attestation与checksum不能替代Windows Authenticode。

## Leads

- 固定v1.1.0；上层实现跨进程全局桌面租约、fencing token、deadline/cancel和crash recovery。
- 另做Windows/macOS用户输入hook并区分synthetic事件，任何用户输入暂停后续dispatch。
- 默认禁用raw坐标、mouse、Windows press、--force、--cdp；不确定命中或前台身份变化一律fail closed。
- 每动作后独立Observe，delivered_unverified或断连映射unknown且不自动重试。
- 自行签署Windows工件；默认关闭trace/screenshots并由宿主管ACL、配额与清理。

## Looked for but not found

- 未找到全局用户输入立即暂停、跨进程全桌面租约、执行中cancel、每动作强制Observe、高风险确认UI、Windows Authenticode、macOS notarization声明、统一隐私窗口排除、正式威胁模型/SLA。
