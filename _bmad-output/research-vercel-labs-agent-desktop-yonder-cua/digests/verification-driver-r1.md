# Driver关键主张独立核验

核验对象：API/架构与平台/安全Round 1摘要；固定`v1.1.0`，访问日期2026-11-04。

## 结果

- **推翻（原主张的一部分）**：`v1.1.0`已经有machine-wide、跨进程interaction lease。Windows使用token-derived独占锁，两个进程不能交错focus/click/type；Unix/Windows均有对应源码实现。来源：[Windows权限说明](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop-windows/references/permissions-and-elevation.md)、[Concepts](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/CONCEPTS.md)、[interaction_lease.rs](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/crates/core/src/session/interaction_lease.rs)。限定：租约只覆盖单次dispatch，不提供任务级fencing/所有权、用户输入抢占或执行中cancel。
- **验证**：未找到用户真实键鼠输入立即暂停或执行中cancel。60个CLI命令、C ABI与源码未见pause/takeover/cancellation或Windows hook/macOS event tap。来源：[C ABI](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/crates/ffi/include/agent_desktop.h)及上述官方命令文档。
- **验证**：Driver不强制每次action后完整Observe。循环归调用方，只有部分stateful action自动局部回读；完整业务效果仍需调用方独立Observe。来源：[Agent Skill](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/SKILL.md)、[Interaction Commands](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-interaction.md)。
- **验证**：qualified ref与动作前重识别能拒绝STALE_REF/AMBIGUOUS_TARGET，但hit-test `Unknown`明确fail open，raw `--xy`无窗口身份，Windows `press`只写foreground queue。来源：[Concepts](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/CONCEPTS.md)、[Interaction Commands](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-interaction.md)、[Windows Skill](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop-windows/SKILL.md)。独立审计：[Skillstore Audit](https://skillstore.io/skills/vercel-labs-agent-desktop/audits/1)，2026-10-31。
- **验证**：Windows官方release未Authenticode签名。官方文档明确说明，核验者下载PE确认Security Directory为零；SHA/attestation不等于签名。来源：[安装文档](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/docs/install.md)、[troubleshooting](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop-windows/references/troubleshooting.md)、[v1.1.0 Release](https://github.com/vercel-labs/agent-desktop/releases/tag/v1.1.0)。
- **验证**：可选`launch --cdp`开放`127.0.0.1` HTTP与WebSocket，属于loopback TCP例外；同用户本地进程可在端口开放时控制Web内容。来源：[System Commands](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-system.md)、[CHANGELOG](https://github.com/vercel-labs/agent-desktop/blob/main/CHANGELOG.md)。

采用建议应删除“补跨进程dispatch锁”，改为“复用并验证现有锁，同时补任务级fencing/所有权、用户输入抢占、cancel和强制action→Observe”。
