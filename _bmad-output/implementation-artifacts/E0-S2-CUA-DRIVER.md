# E0-S2 CUA Driver 对照验证

## Story

作为 Yonder 开发团队，我们需要以同一套无 Agent 黑盒调用对照当前 Qwen CUA SDK 与 trycua CUA Driver，以选择唯一的本地 CUA 执行引擎。

## 候选

- A：`@qwen-code/cua-sdk@0.20.5`
- B：`@trycua/cua-driver@0.25.0`
- 参考但不参选：`@qwen-code/open-computer-use@0.2.3`（旧 MCP 实现，当前 Qwen Code 已迁移）

## 验收条件

1. 首版在同一台 Windows 设备上使用相同应用、动作、观察与故障用例；macOS 对等验证移入后续 Epic。
2. 只测试 Driver/SDK，不接入任何 Agent、模型或 Planner。
3. 记录 API 覆盖、元素定位、截图、延迟、资源、权限、取消、中断恢复、部署大小与许可证。
4. WPS、普通权限文本应用与 Windows 文件资源管理器任务分别产生可复现证据；WPS 作为首版办公套件代表。
5. 任一候选不能在 Windows 安全执行首版核心用例即淘汰；不得长期维护双栈。

OpenSpec：`openspec/changes/e0-compare-cua-drivers/`
