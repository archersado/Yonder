# CX-S2 架构设计

## 边界与依赖

Application拥有一次圈选请求的状态和临时附件生命周期；Windows/macOS平台Adapter只提供区域选择、区域截图和坐标换算。Desktop组合根展示临时覆盖层与确认条。截图Adapter不得直接调用语音、Gateway、任务存储或CUA Adapter。

语音通过VI-S1 Application用例组合；提交给Agent需要新增可信双向Gateway契约。Agent随后使用既有`task.create`创建任务，Yonder不得从UI伪造Agent身份。

## 状态与契约

完整 Story 状态为`idle → selecting → reviewing → submitting → submitted|cancelled|failed`。Application 是会话状态和临时附件生命周期的唯一所有者；Desktop 只转发显式入口与交互事件，平台 Adapter 只返回截图结果或稳定错误。`selecting` 接受框选或笔画；笔画只在选择层内临时绘制，并以外接矩形调用既有区域截图能力，不持久化轨迹。同一设备最多一个圈选会话。

Accepted AD-CX-01 首先授权 macOS Preview 使用`idle → selecting → capturing → reviewing → cancelled|failed`子集；已归档的附件提交Change随后补齐`submitting → submitted|failed`。不创建受控临时文件；截图字节、笔画和选区只在有界内存中存在。关闭、Esc、取消、30秒超时、重新圈选、捕获失败和进程退出都必须让 Application 释放会话及截图字节，Desktop 随后隐藏选择层或确认卡。

未形成有效区域或截图权限不可用时，Application进入无附件`reviewing`，确认卡允许非空文字通过现有`agent.input`提交；`source=selection`表示来自圈选提问入口，不承诺存在附件。该路径不要求`user_input_attachment`能力，不得生成空附件、伪造截图成功或因附件能力缺失拒绝文字。

Preview 进入`selecting`前只读检查CUA前台租约；租约已占用时返回`desktop-control-active`并保持`idle`，不显示覆盖层，也不调用CUA Driver。该拒绝只验证 CX2-07 的“不争夺指针”部分；完整 Story 的“先暂停任务”仍须在后续 Change 接入既有用户输入停止语义后验证。

### Agent 临时附件候选

现有`agent.input`仅承载有界文字，不能内嵌截图或发送本机路径。Accepted AD-CX-02规定在同一已认证`AgentSession`上先传输单个不超过4 MiB的会话级附件，再由`agent.input`引用其`attachment_id`。本地与云端共用Rust协议、64 KiB帧上限、哈希校验、deadline和清理语义；任务库、事件、Outbox和日志不保存截图、正文或哈希。隔离Spike、产品接线及独立复核均已通过并归档。

## 双平台原生路线

使用系统原生显示器、缩放和截图能力；覆盖层仅在显式圈选期间存在，不迁移参考插件的Electron全屏模式和`ptrtap`常驻进程。全局快捷键优先使用系统注册热键，不以持续输入监听实现。

macOS ScreenCaptureKit/CoreGraphics候选与Windows Graphics Capture候选必须使用同一多显示器样本验证：逻辑点/物理像素换算、DPI、负坐标、窗口/显示器变化、权限撤销和异常清场。路线定案前建立Architecture Decision。

## 失败与验证

系统安全界面、密码管理器、用户排除应用和受保护内容拒绝捕获。屏幕权限撤销、显示器断开或覆盖层崩溃必须fail closed并恢复输入。外部发送需要用户确认，unknown不自动重试。

## 架构影响

该Story新增显式屏幕区域上下文和用户向Agent发起请求的边界。AD-CX-01 已接受 macOS 单显示器 Preview，AD-CX-02及产品Change已接受临时附件提交；均不改变持久化、依赖方向或状态所有者。无附件文字提交复用既有`agent.input`协议，属于conforming增量。语音组合、跨显示器和Windows仍须完成各自门禁后另建 Change。
