# AG-S8 视觉与交互设计

## 入口与流程

用户安装 Yonder 提供的 Codex Hook 后，在 Codex 的 Hook 信任界面审阅并确认。Yonder 不静默修改 Codex 全局配置，也不要求用户复制技术 ID 或 Socket。

## 状态与错误反馈

- 已连接：Yonder 既有 Agent 连接状态显示“Codex CLI 已连接”；任务面板和顶部执行条继续使用统一任务状态，不增加第二套状态。
- 正在恢复：连接器重连期间不得显示已连接；若已有通用连接状态可显示“Codex CLI 正在重新连接”。
- 未连接：Hook 未安装/未信任、恢复窗口耗尽、Codex 不兼容时显示“Codex CLI 未连接”，诊断只给原因类别和修复动作。
- ChatGPT Desktop：入口明确显示“当前不支持”，不得出现可点击但无效的连接按钮。

## 无障碍与平台验证

状态不可只依赖颜色；使用文本与图标共同表达。所有设置和诊断支持键盘访问及 VoiceOver。错误信息不包含输入正文、完整 session ID、完整 endpoint 或 Payload。
