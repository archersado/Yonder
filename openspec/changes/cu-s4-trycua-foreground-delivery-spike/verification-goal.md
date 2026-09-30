# 独立 Verification Goal：trycua 单次前台投递升级 Spike

状态：PASS（macOS Spike子范围；Windows暂缓，产品Apply未开始）

复核目标：候选版本与Schema冻结；精确窗口前台click；焦点与固定标记双重Observe；原前台恢复；错误目标和不可核实结果fail-closed；关闭无残留；证据不含敏感内容。

通过条件：所有macOS自动样本通过且证据字段完整。Windows按用户决定暂缓，因此本Goal通过也只代表macOS Spike子范围，不代表产品Apply或完整跨平台Story完成。

## 2026-09-30 结果

- 0.30.4工具目录确认`click/hotkey/type_text`均含精确target、窗口坐标和`delivery_mode`；SDK元数据、`callTool`及清理可用。
- 0.25.0与0.30.4统一样本均通过：错误窗口拒绝且目标未变化；原子前台输入为confirmed；SDK Observe与原生控件值一致；诱饵前台恢复；shutdown后动作拒绝；fixture进程清理。
- 原前台进程在动作前失效的负样本没有派发输入，结果失败且进程清理，证明恢复前置失效会阻断成功。
- 独立`verify.py`只读取有界JSON并复核版本、正负样本和隐私字段，输出`upgrade_required=false`。

结论：macOS Spike PASS，但0.30.4无独占收益，不升级。AD-CU-07接受保留0.25.0并由后续独立Apply Change接入受限前台坐标文本能力。
