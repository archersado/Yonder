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

关联Accepted [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-TRYCUA-FOREGROUND-DELIVERY.md)。Spike阶段Architecture Impact为`none`；产品接线通过独立Apply Change修改协议与Adapter Worker，不替换SDK版本。

## 产品Apply（协议1.39）

Accepted AD-CU-07决定保留0.25.0。Rust协议与Application只为`enter-target-query`、`draft-message-ref`的`type_text`接受恰好两个有限坐标参数；既有视觉`click`仍只接受同样的`x/y`。`delivery_mode`、target、正文和会话身份不得来自Agent。

Worker在展开意图引用后，为坐标文本或点击注入SDK工具目录声明的`delivery_mode=foreground`、精确窗口target和受监管session。坐标文本是单个Driver动作，不先发独立click；动作后Observe同一窗口并保留截图。只有`confirmed + observe_valid`可推进，其他结果交回且不得建立视觉焦点凭据或重试。

## AX优先与视觉降级补充（AD-CU-08）

Worker首次Observe仍以`include_screenshot=false`读取已绑定窗口的AX元素。若元素树为空，或动作结果不是`confirmed`，只对同一可信PID/window补采一次截图；正常元素路径不增加截图成本。该截图是当前失败边界的新鲜Observation，不触发动作重放，也不改变任务结论。

`UnknownObserved`携带的Observation必须由TM-S9内存运行时原样返回Gateway，不能因为持久化异步化而丢弃。通用`computer.step`坐标文本复用同一精确窗口foreground注入；受保护消息计划仍从内存引用展开正文并保留发送确认，不因通用搜索场景放宽。

## 通用桌面计划片段补充（协议1.40）

通用桌面任务由归属慢脑从同一Gateway一次提交1～10个槽位。协议新增`focus-control`、`input-text`、`activate-control`三种封闭动作语义：聚焦只允许窗口内点击或`cmd+f`，输入只允许有界文本与可选窗口局部坐标，激活只允许窗口内点击或`ENTER/RETURN/SPACE`。Application在派发前复用同一参数校验，并把语义标记注入Worker；Agent不能提交Driver身份、session、target或投递模式。

计划接受后，宿主从已验证片段建立全部槽位的只读内存投影，顶部浮窗展示片段总数与当前附近至多四个槽位标签；执行只移动当前槽位和完成状态，不能用`computer.step`的“慢脑单步”覆盖整个片段。每步后仍Observe；AX事实不足才走同窗口视觉降级，越界或失败则交回同一归属慢脑重规划。
