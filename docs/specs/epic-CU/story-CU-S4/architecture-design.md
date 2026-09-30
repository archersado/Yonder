# CU-S4 架构设计

## 边界与候选

Spike在`spikes/cua-foreground-delivery/`中运行，固定比较产品当前0.25.0与候选0.30.4。两者均只通过官方进程内SDK读取工具Schema和运行隔离样本；不连接Yonder Gateway、不读任务库、不复用产品Worker进程。

0.30.4静态契约新增`ClickInput.target + position + deliveryMode`，其中`Foreground`是本Spike唯一新增候选；`hotkey`与`type_text`仍保持精确target但没有独立`deliveryMode`。因此样本必须验证前台click是否真实建立焦点，以及随后target-bound文本输入是否落在同一输入框，不能从类型存在推导可用。

## 统一样本

隔离fixture包含一个可读焦点和值的文本框，并另开一个诱饵窗口。探针记录原前台身份，取得目标PID/窗口和新鲜窗口截图，从截图坐标执行一次前台click，随后输入固定标记`YONDER_FGD_SAMPLE`，再以独立原生控件读和SDK Observe双重验证。最后检查原前台恢复、诱饵未变化、Worker关闭和进程清理。

失败注入覆盖错误窗口、过期截图或坐标、目标关闭、`unverifiable`和动作后Observe失败。任何失败停止样本，不重放点击或输入。

## 数据与依赖

证据只保存版本、Schema字段布尔、效果枚举、焦点/值匹配布尔、前台恢复布尔、耗时和进程清理布尔；不保存截图、窗口标题、用户输入或完整SDK Payload。Spike依赖独立package-lock，不修改`apps/desktop/cua`。

## 淘汰门槛

缺少以下任一项即淘汰升级路线：固定包可验证安装、精确窗口前台click、明确焦点后置事实、同窗口文本Observe、原前台恢复、错误目标fail-closed、关闭清理。只在fixture通过而真实自绘应用仍无确认时，结论缩小为“原生fixture兼容”，不得解除EX-S2正式样本门禁。

## 架构影响

关联Proposed [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-TRYCUA-FOREGROUND-DELIVERY.md)。Spike阶段Architecture Impact为`none`；通过后若替换产品版本，必须先接受ADR并建立独立Apply Change，单版本迁移Adapter/Worker/打包与验证。
