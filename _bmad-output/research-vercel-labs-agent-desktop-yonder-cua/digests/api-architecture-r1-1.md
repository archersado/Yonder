# API与架构研究摘要（Round 1）

研究者结论：`agent-desktop`接近可抽取的跨平台CUA Driver内核，不是完整Agent；适合作为限时Spike候选，不可原样视为满足受监管产品要求。

## Findings

1. `{claim: "Rust core定义PlatformAdapter和共享类型，macOS/Windows/Linux crate分别接AX、UI Automation、AT-SPI；上层提供CLI与cdylib C ABI。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/AGENTS.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "源码架构"}`
2. `{claim: "入口包括一次性CLI、进程内C library和MCP stdio；公开主入口不要求HTTP/TCP daemon或P2P。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "接口"}`
3. `{claim: "Observe包含snapshot/find/get/is/screenshot/list-surfaces；snapshot返回accessibility tree、snapshot_id、complete和snapshot限定ref。动作覆盖ref动作、原始键鼠、应用窗口、wait、session/batch；截图不是每次action必带。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/README.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "API"}`
4. `{claim: "qualified ref携带snapshot id，动作前严格重识别；缺失、陈旧、歧义fail closed。窗口身份组合handle/window number/client id与进程创建代际token，抵抗句柄复用和标题漂移。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/CONCEPTS.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "身份安全"}`
5. `{claim: "C ABI支持进程内抽取：ad_snapshot生成树和qualified refs，ad_execute_by_ref执行动作；native handle线程亲和，桌面mutation由interaction lease串行化。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/crates/ffi/include/agent_desktop.h", publisher: "vercel-labs/agent-desktop", pub_date: "ABI major 7/main", accessed: "2026-11-04", confidence: "高", class: "C ABI"}`
6. `{claim: "官方明确工具不是Agent，不内置LLM或Agent loop；headless只走语义accessibility，headed才允许focus/cursor/physical fallback。interaction lease仅覆盖dispatch，取得lease后重新解析目标。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/AGENTS.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "边界并发"}`
7. `{claim: "可选launch --cdp会为Chromium应用建立DevTools endpoint，可能引入localhost HTTP/WebSocket/TCP；若禁止本地TCP需裁剪。未发现P2P。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/CHANGELOG.md", publisher: "vercel-labs/agent-desktop", pub_date: "2026-08-14", accessed: "2026-11-04", confidence: "高", class: "网络边界"}`
8. `{claim: "错误envelope提供稳定code/message/disposition，以及delivery与retry分类；变更超时若可能已落地不会宣称safe，支持PERM_DENIED、STALE_REF、AMBIGUOUS_TARGET、POLICY_DENIED、ACTION_FAILED、TIMEOUT等。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/docs/json-output.md", publisher: "vercel-labs/agent-desktop", pub_date: "文档envelope 2.5，README示例2.6", accessed: "2026-11-04", confidence: "高但有文档漂移", class: "错误语义"}`
9. `{claim: "snapshot超预算返回ok:true/complete:false/truncated:true；mutating timeout的delivery可为unknown。未发现正在执行调用的task cancel、断连恢复或统一interrupted协议。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/docs/json-output.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "超时缺口"}`
10. `{claim: "部分有状态ref action会在单次投递后检查postcondition contradiction并返回ACTION_FAILED，但不会统一返回完整新snapshot，调用方仍须强制action后Observe。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/docs/json-output.md", publisher: "vercel-labs/agent-desktop", pub_date: "对应2026-09-12变更", accessed: "2026-11-04", confidence: "高", class: "Observe语义"}`
11. `{claim: "macOS原生E2E操作真实fixture并用独立accessibility observation验证效果；覆盖headless/headed、超时、单次投递和同名双窗口身份。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/tests/e2e/README.md", publisher: "vercel-labs/agent-desktop", pub_date: "main检索态", accessed: "2026-11-04", confidence: "高", class: "测试证据"}`
12. `{claim: "2026-11-01至03开放issue报告Windows窗口漏报、Word不可观测、macOS ref/window identity误判、Linux AT-SPI卡死、混合缩放错误与RefMap超限，跨应用可靠性仍有限。", source: "https://github.com/vercel-labs/agent-desktop/issues", publisher: "项目贡献者", pub_date: "2026-11-01至03", accessed: "2026-11-04", confidence: "中高，部分未确认维护者复现", class: "兼容性风险"}`

## Leads

- 固定release后审计deadline、lease与FFI；优先C ABI而非每命令CLI。
- 编译/策略禁用CDP并审计监听端口。
- 补宿主request_id/deadline/cancel、用户输入抢占、进程隔离、unknown映射与每步强制snapshot。
- 双平台统一样本覆盖同名窗口、句柄复用、缩放、多屏、权限拒绝、超时和用户抢占。

## Looked for but not found

- 未找到任务级cancel/stop current action或cooperative cancellation。
- 未找到用户真实键鼠输入立即暂停机制。
- 未找到stdio断连后的可查询interrupted/恢复协议。
- 未找到每个action强制完整Observe规则。
- 未找到Windows/macOS同等强度的持续真实桌面E2E公开证据。
- 未找到P2P；核心Driver未发现必须依赖TCP，CDP为明确例外。
