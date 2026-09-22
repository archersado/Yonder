# EX-S4 架构设计

## 边界与依赖

Application 只从当前计划和授权范围构造完整的 Document/Command 候选，再由 Jev 选择「使用哪一项」或交回。文档语义修改沿用现有 Document Port/OOXML Adapter、FI 文件身份与锁；Office UI 编辑属于单独显式 CUA 步骤。Command 沿用结构化 `program + args + cwd + env`，不得拼接 Shell 字符串。Jev 不能补填文档正文、文件路径、命令参数或隐式选择覆写；参数缺失、多个同名文件、锁冲突、哈希变化直接返回外部慢脑/用户可见错误。

## 状态与契约

执行准入、敏感操作确认、单文件写锁、expected_hash、另存/覆盖选择、输出限额、超时及进程树停止均由既有用例/Adapter 负责。快脑只能在这些闸门之前选候选，不能修改结果。副作用后 Observe 文档哈希/结构或命令退出与允许的结果摘要；`unknown` 不自动重试，事件与 Outbox 同事务。不要为 Office 和 Command 各建一套快脑循环。

## 失败与验证

用隔离 DOCX/XLSX/PPTX 与无副作用命令做成功、锁冲突、旧哈希、参数缺失、超时及取消对照；真实 Office/WPS 锁、Windows/macOS 原生证据仍按 DO/FI/CM 各自门禁。EX-S1 的收益对照不合格时不接此子范围。
