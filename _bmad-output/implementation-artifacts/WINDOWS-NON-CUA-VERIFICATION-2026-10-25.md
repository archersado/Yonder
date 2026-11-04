# Windows 非 CUA 验证进展

日期：2026-10-25

## 本次新增通过

- BU-S1 Windows ego-lite：修复为 PowerShell 5.1 兼容的 UTF-8 `.mjs` + ASCII 文件 URL import 启动方式；ego-lite `0.6.1.11` 的 create、observe、handOff、takeOver、finish 全部通过。
- BU-S1 持久化：外部引用跨 SQLite 重开可恢复，finish 状态持久化且 Task Space 已清理。
- BU-S1 独立 Verification Goal：Windows Adapter 35/35、Windows Workspace 77/77、架构检查 17/17、OpenSpec 严格校验及相关 diff 检查全部通过。
- DS-S1 Windows 自动空闲停靠：修复启动状态读取竞态；两个无输入样本分别在181.401秒和181.210秒进入底边112×56与左边56×112，关闭2026-10-24自动触发失败项。
- BU-S2 Windows完整技术链：正式Named Pipe Gateway与真实ego-lite完成create、引用读取、hand-off、take-over、finish及桌面重启后恢复；任务`completed@18`，4个Observed attempt，Task Space引用UI与局部失败通过。Windows产品目标76/76通过，未执行CUA。

## 仍未通过或受外部条件阻塞

- DS-S1 完整 Story：自动空闲停靠子范围已通过；顶部/右侧、点击/键盘恢复、拖动、九态完整原生链路、性能和 macOS 剩余证据仍未完成。
- EX-S1 Jev：缺少真实服务凭据/密钥，只完成本地自检。
- Windows 原生语音录制：仍受 Spike 门禁约束；现有降级反馈已通过。
- AG-S5 产品输入链：仍需真实已登记且支持输入确认的 Agent Runtime 完成端到端验证。
- BU-S2 技术验证已双平台通过，仅剩PR与Archive流程，不再列为Windows功能阻塞。

本轮未执行或修改 CUA。
