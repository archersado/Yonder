# Yonder 产品简报补充材料

## ego-lite 接入参考

资料来源：

- https://github.com/citrolabs/ego-lite
- https://lite.ego.app/document/en/docs/ego-browser
- https://lite.ego.app/document/en/docs/custom-agent-harness

ego-lite 采用“Agent Skill + 本地主机 CLI + 应用内 Runtime”：Skill 指导 Agent 使用工具，`ego-browser` CLI 是通用调用入口，桌面应用负责浏览器状态、Snapshot、Task Space、事件和用户接管。它不以 MCP 为主要兼容机制；沙箱或云端 Agent 需要 host bridge 才能触达本机。

Yonder 应采用可安装的 Agent Skill 与本地 CLI/Runtime 契约，将 MCP 作为同一 Runtime 上的可选适配器。与 ego-lite 不同，Yonder 桌面插件还需要接入外部平台提供的、经过认证的云端到设备通道；Yonder 不实现云端服务端。无论任务从哪里发起，权限、中断和敏感操作确认都由本地 Runtime 执行。

## CUA 与 BUA 的 Task Space

Task Space 是一个 Agent 任务的持久容器，包含身份、发起 Agent、授权范围、生命周期、资源引用、事件时间线、产物和审计记录。用户和 Agent 均可查看、暂停、恢复、取消与交接任务。

- BUA 在浏览器上下文能够隔离时支持并行。
- CUA 使用逻辑 Task Space，但不承诺操作系统级隔离；MVP 串行执行前台鼠标和键盘操作。
- 窗口、标签页和文件由 Task Space 引用，所有权仍属于原应用。

## CUA 开源实践与候选后端

资料来源：

- https://github.com/QwenLM/qwen-code/blob/main/docs/users/features/computer-use.md
- https://github.com/QwenLM/qwen-code/tree/main/packages/cua-driver
- https://github.com/QwenLM/open-computer-use
- https://github.com/trycua/cua
- https://github.com/xlang-ai/OSWorld
- https://github.com/bytedance/ui-tars-desktop
- https://github.com/microsoft/UFO

### 执行原则

- Yonder 只需要与模型无关的本地执行引擎，不引入 Agent、规划器、模型循环或 Qwen Code Runtime。
- 感知优先使用 Accessibility Tree，截图作为回退；执行采用“观察—操作—验证”循环。
- 前台焦点、全局鼠标、键盘注入、安全输入框与破坏性动作分别设闸。
- Task Space、权限、云端桥接、Record 时间线和审计留在 Yonder 层，确保底层驱动可替换。

### 候选对比

**Qwen `cua-driver`：** 当前候选。提供 Windows/macOS 原生驱动、窗口状态、带稳定元素索引的 Accessibility Tree、截图、输入、Session、录制与轨迹回放。需要验证能否脱离 Qwen Code 独立复用，以及许可证、签名和 API 稳定性。

**trycua `cua-driver`：** 对照候选。在 macOS 上具备后台执行、非 Accessibility 界面支持和可回放轨迹；跨平台 Rust 版本仍需重点验证 Windows 成熟度。

**Qwen `open-computer-use`：** 作为轻量 MCP/CLI 协议与打包参考。它提供较小的跨平台 Accessibility-first 工具集；若无法直接复用 `cua-driver`，可作为回退候选。

**OSWorld：** 属于可复现桌面环境与 Agent 评测框架，依赖虚拟机、容器或云端环境完成任务复位、执行和评分，不适合作为用户真实电脑上的产品执行引擎。后续可用于基准测试、回归测试和隔离的破坏性任务验证。

UI-TARS 可参考截图驱动的视觉回退与远程事件流；Microsoft UFO 可参考 Windows UIA、Win32、COM 和 Office 专用动作。两者的完整 Agent 框架均不进入 Yonder。

### 验证方案

Qwen 与 trycua 使用同一套 Windows/macOS 黑盒用例，对比：

- 安装、签名和系统权限；
- Word、Excel、PowerPoint、WPS、Finder 和文件资源管理器覆盖；
- Accessibility Tree 完整性与元素稳定性；
- 后台操作、坐标缩放、Unicode 输入和用户接管可靠性；
- Session、录制与 Replay 原语；
- 许可证与再分发条件。

验证完成后 MVP 只选择一个后端，不维护双驱动长期共存架构。
