# CX-S2 架构设计

## 边界与依赖

Application拥有一次圈选请求的状态和临时附件生命周期；Windows/macOS平台Adapter只提供区域选择、区域截图和坐标换算。Desktop组合根展示临时覆盖层与确认条。截图Adapter不得直接调用语音、Gateway、任务存储或CUA Adapter。

语音通过VI-S1 Application用例组合；提交给Agent需要新增可信双向Gateway契约。Agent随后使用既有`task.create`创建任务，Yonder不得从UI伪造Agent身份。

## 状态与契约

候选状态为`idle → selecting → reviewing → submitting → submitted|cancelled|failed`。`selecting` 接受框选或笔画；笔画只在选择层内临时绘制，并以外接矩形调用既有区域截图能力，不持久化轨迹。同一设备最多一个圈选会话；进入`selecting`前检查CUA前台租约并触发现有用户接管/暂停机制。截图与转写只保存在有界临时内存或受控临时文件，完成后清理。

## 双平台原生路线

使用系统原生显示器、缩放和截图能力；覆盖层仅在显式圈选期间存在，不迁移参考插件的Electron全屏模式和`ptrtap`常驻进程。全局快捷键优先使用系统注册热键，不以持续输入监听实现。

macOS ScreenCaptureKit/CoreGraphics候选与Windows Graphics Capture候选必须使用同一多显示器样本验证：逻辑点/物理像素换算、DPI、负坐标、窗口/显示器变化、权限撤销和异常清场。路线定案前建立Architecture Decision。

## 失败与验证

系统安全界面、密码管理器、用户排除应用和受保护内容拒绝捕获。屏幕权限撤销、显示器断开或覆盖层崩溃必须fail closed并恢复输入。外部发送需要用户确认，unknown不自动重试。

## 架构影响

该Story新增显式屏幕区域上下文和用户向Agent发起请求的边界。Proposed AD-CX-01及Architecture Spine候选边界已建立，只授权隔离Spike；Windows与macOS同一样本通过并接受ADR后，才能生成产品实施OpenSpec。
