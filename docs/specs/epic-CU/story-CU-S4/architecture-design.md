# CU-S4 架构设计

## 边界与候选

Spike在`spikes/cua-foreground-delivery/`中运行，固定比较产品当前0.25.0与候选0.30.4。两者均只通过官方进程内SDK读取工具Schema和运行隔离样本；不连接Yonder Gateway、不读任务库、不复用产品Worker进程。

0.30.4原生类型新增`ClickInput.target + position + deliveryMode`；实际`listToolsJson/callTool`目录进一步为`click`、`hotkey`和`type_text`公开精确target、窗口坐标与`delivery_mode`。Spike优先验证`type_text(x,y,text,foreground)`能否原子建立焦点并输入，同时保留分离click路径的Schema证据；不能从字段存在推导可用。

## 统一样本

隔离fixture包含一个可读焦点和值的文本框，并由独立进程打开诱饵前台窗口。探针记录诱饵身份，取得目标PID/窗口和新鲜窗口截图，从截图坐标执行一次原子前台输入固定标记`YONDER_SDK_INPUT_A`，再以独立原生控件读和SDK Observe双重验证。最后检查诱饵前台恢复、诱饵未变化、Worker关闭和进程清理。

失败注入覆盖错误窗口、过期截图或坐标、目标关闭、`unverifiable`和动作后Observe失败。任何失败停止样本，不重放点击或输入。

## 数据与依赖

证据只保存版本、Schema字段布尔、效果枚举、焦点/值匹配布尔、前台恢复布尔、耗时和进程清理布尔；不保存截图、窗口标题、用户输入或完整SDK Payload。Spike依赖独立package-lock，不修改`apps/desktop/cua`。

## 淘汰门槛

缺少以下任一项即淘汰升级路线：固定包可验证安装、精确窗口前台click、明确焦点后置事实、同窗口文本Observe、原前台恢复、错误目标fail-closed、关闭清理。只在fixture通过而真实自绘应用仍无确认时，结论缩小为“原生fixture兼容”，不得解除EX-S2正式样本门禁。

## 架构影响

关联Proposed [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-TRYCUA-FOREGROUND-DELIVERY.md)。Spike阶段Architecture Impact为`none`；通过后若替换产品版本，必须先接受ADR并建立独立Apply Change，单版本迁移Adapter/Worker/打包与验证。
