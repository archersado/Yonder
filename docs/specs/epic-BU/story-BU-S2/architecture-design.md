# BU-S2 架构设计

## 边界与依赖

Application定义BrowserReferenceRecord并扩展TaskStore Port；Adapter在既有attempt结果事务内写当前引用。BU Adapter不访问SQLite，Desktop/UI不成为引用所有者。

## 状态与契约

schema12新增每任务一行的task_browser_refs，包含受限`ego:<数字>`引用、所有权、托管页数量、finished和updated_sequence。updated_sequence指向同次running→running结果事件。后续结果覆盖当前行，历史attempt与事件仍保留审计。

## 失败与验证

非法引用、unknown、旧身份、CAS或Outbox失败全部回滚；迁移前备份，未知格式拒绝。真实macOS Bridge结果写入SQLite并重开读取；Windows暂缓。

## 架构影响

改变持久化与TaskStore Port，依据Accepted AD-BU-01实施；不新增外部协议或依赖方向。

## Browser引用只读详情（2026-09-18）

依据Accepted AD-BU-03，协议1.15复用`GetParams`和`task.read`增加`task.browser.get`，返回现有安全`BrowserReference`或明确缺失，不改SQLite。Gateway按版本门禁；本机TaskHost使用可信LocalUser。读取不启动Bridge、不取得Browser资源、不改变ego-lite所有权。

2026-09-18用户修订：打开入口使用可信桌面命令，UI只传`task_id + expected_sequence`；TaskHost不接受UI提供外部引用，通过现有Browser用例重新读取引用并执行`hand-off`。该动作沿用Browser资源、attempt、Observe及事务提交；Agent侧`take-over`仍走同一Gateway执行链。
