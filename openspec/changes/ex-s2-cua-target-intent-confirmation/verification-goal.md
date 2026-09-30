# EX-S2 CUA目标意图与发送确认 Verification Goal

状态：自动化通过，待正式macOS产品样本。Windows按用户决定暂缓，不Archive。

验证必须覆盖：协议不回显敏感值、内存容量/期限/任务归属、动作语义映射、元素与视觉候选、顶部确认、批准一次消费、拒绝/过期/unknown不重试，以及正式Yonder企业微信多步骤样本。

## 2026-09-30 自动化结果

- Rust协议、Application、Adapter、CLI及Desktop共188项测试通过；Adapter全量首次运行有1项既有文件系统用例瞬时失败，单项立即复跑通过。
- 隔离Worker样本完成6个元素语义动作；视觉fixture确认封闭`⌘F`搜索动作、精确窗口target、只有`confirmed`聚焦建立一次性凭据且只消费一次，`unverifiable`聚焦后的引用文本被拒绝，私有字段在调用SDK前剥离；元素歧义不执行副作用。
- OpenSpec严格校验、Rust派生Schema检查、Node/Python语法检查通过。全仓架构检查被既有`TM-S9`三份设计缺少标准Story字段/章节阻断，与本Change无关，未据此放宽门禁。
- 正式签名macOS Yonder的企业微信样本仍待下一步运行，未据自动化结果提前Archive。

## 2026-09-30 正式产品样本结果

- 通过安装包内`yonder mcp`与正式Gateway创建两条企业微信任务；任务、计划步骤、交回状态与临时窗口截图均由Yonder产生，未直连Socket、Driver或SQLite。
- `launch_app`确认成功；固定trycua 0.25.0对企业微信精确窗口的后台`⌘F`、坐标点击和引用文本输入均返回`unverifiable`，同次Observe有效但截图未出现搜索文本，因此未发送消息。
- `bring_to_front`后的截图显示窗口由非活动转为活动，但其结构化结果未满足当前精确窗口确认契约，Yonder仍按交回处理。
- 样本证明“`unverifiable`聚焦后允许引用文本续跑”不安全，已改为只有`confirmed`聚焦才建立一次性凭据；两条验证任务均已通过Gateway取消，未遗留运行中副作用。
- macOS正式样本未通过，Change不得Archive。后续须以独立Spike验证新版trycua的单次前台投递并恢复能力，或修正0.25.0激活结果映射；在取得确认后置事实前不得恢复引用文本续跑。
