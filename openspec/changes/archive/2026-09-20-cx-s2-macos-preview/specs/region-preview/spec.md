# 圈选提问 macOS Preview 增量规格

## ADDED Requirements

### Requirement: 显式且短生命的区域选择

系统必须只在用户明确启动后于当前显示器提供一次区域选择，并在结束时清理临时截图。

#### Scenario: 框选或笔画成功选择

- Given macOS屏幕捕获权限可用且没有CUA前台租约
- When 用户选择框选后拖出有效区域，或选择笔画后画出有效笔画并松开
- Then 笔画以其外接矩形作为截图范围
- Then 系统显示本地确认卡，截图只在内存保留且不会发送或持久化

#### Scenario: 取消或拒绝

- Given 用户按Esc、取消、超时、权限缺失或CUA正在控制桌面
- When 圈选会话结束
- Then 系统清理选择层、选区、笔画和截图字节，不创建任务、不发送Agent输入
- Then 再次打开时不得显示上一会话的预览

#### Scenario: CUA租约拒绝

- Given CUA前台租约已被一个运行中任务持有
- When 用户从任一Preview入口启动圈选
- Then 系统返回`desktop-control-active`并保持空闲，不显示选择层、不暂停或驱动CUA
