当前归属 Story：CX-S1；规划：`docs/specs/epic-CX/story-CX-S1/README.md`。旧编号保留历史追溯。

# 设计

前台窗口探针直接调用 Win32 API，仅输出进程标识、应用名以及窗口标题是否存在和长度，验证证据不落正文。正式实现改为 WinEvent Hook 的事件驱动 Adapter。

Native Messaging Host 使用标准输入输出的长度前缀 JSON 帧；测试覆盖 UTF-8、分片、多帧和超限拒绝。Chrome/Edge manifest 仅接受精确扩展来源，使用 HKCU 注册，且扩展不申请 History 权限。

操作序列钩子只允许存在于显式 Recording 生命周期中。Spike 由用户按 Enter 开始，在有界时间后通过 `finally` 解除低级键盘和鼠标 Hook；只记录事件数量，不记录键值、坐标或正文，不预建事件模型和持久化层。
