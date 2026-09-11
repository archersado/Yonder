# 设计

Harness 以固定 JSON 用例直接驱动两个 SDK，不包含 Agent 或规划逻辑。两者当前公开 API 与 Contract 相同，因此不预建两个空壳 Adapter；只有字段实际分叉时才增加最薄映射。每一步保存请求、脱敏响应、耗时、进程资源和预期条件结果。

首版核心用例：Windows 列举应用/窗口、获取 AX 树与截图、按元素点击、文本输入、取消、Driver 崩溃后重新观察。办公任务以 WPS 代表办公套件，并覆盖 Windows 文件资源管理器。Microsoft Office 与 macOS/Finder 对等验证移入后续 Epic。

候选按版本固定。安装脚本执行前必须检查包清单、完整性、许可证和下载目标；不得直接执行远程 shell。结论只允许 Qwen、trycua 或两者均淘汰。
