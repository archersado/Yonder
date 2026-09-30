# 独立 Verification Goal：CU-S4受限前台坐标输入

状态：待实施后运行

复核协议1.39参数白名单、Worker精确target与foreground注入、引用不进入计划/日志、confirmed+Observe推进、失败交回、顶部步骤状态以及正式Yonder企业微信样本。Windows按用户决定暂缓，不据macOS结果宣称跨平台完成。

## 自动化结果

协议、Application、Adapter、CLI与Desktop共188项Rust测试通过；Worker隔离fixture确认坐标click和坐标引用文本均由Worker注入`foreground + exact target`，Agent私有字段在SDK调用前剥离，confirmed与Observe有效才成功，未确认焦点不能消费引用。OpenSpec严格校验、Node/Python语法与diff检查通过。

正式macOS Yonder企业微信样本仍待构建后运行，因此当前Goal未PASS，不Archive。

## 2026-09-30 正式企业微信样本

已从签名后的 `Yonda.app/Contents/MacOS/yonder mcp` 创建真实任务，并经同一 Gateway 执行「启动企业微信→交回→重绑企业微信可信窗口→前台坐标输入」。协议 1.39、私密意图引用、unknown 交回、新片段 step/attempt、窗口重绑和窗口截图均生效；交回后不再误用当时的前台 VS Code。

trycua 0.25.0 对企业微信搜索框的中文与 ASCII 两组原子坐标输入均返回 `effect=unverifiable`，无进一步 escalation reason；同次窗口截图确认文本没有落入搜索框。Yonder 将两次结果保留为 `unknown(observe-failed)` 并交回，没有建立焦点凭据、跨越槽位或发送消息。故自动化合约通过，但本 Goal 的真实企业微信正向样本仍未 PASS，不 Archive。
