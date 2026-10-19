# EX-S2 CUA目标意图与发送确认 Verification Goal

状态：macOS正式产品样本通过。Windows按用户决定暂缓。

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

## 2026-10-19 自动续跑与终局核验终局证据

- 正式产品任务`task_43de32dede4e13cadcc5083f03b4a21f`经安装包内MCP、Gateway与七槽位计划完成企业微信敏感发送。一次本机确认后，最后动作在序号29形成`observed(action_succeeded=true, observe_valid=true)`，计划于序号31耗尽；证据不保存收件人或正文。
- 首次回归暴露Codex Agent Bridge仍把握手版本钉在1.51，导致Desktop无法向当前会话投递协议1.53 `goal_verification`输入。修复后Bridge直接使用同发布包Rust协议版本，并以握手测试锁定1.53及既有四项输入能力。
- 无外部副作用任务`task_6b80801d7fb243675082c16a02564fc5`经正式Yonder启动Finder，序号5形成有效Observe、序号7等待终局核验。Connector重连后，终局核验输入真实抵达当前Codex活动会话；慢脑重新读取Gateway事实，于序号8核验达成、序号9完成，顶部浮窗随终态收起。
- Codex Bridge 18/18、协议输入门禁、Desktop终局投递与浮窗状态测试均通过。结构化证据见`apps/desktop/evidence/ex-s2-goal-verification-wake-macos-20261019/`。

结论：macOS范围的意图短期引用、发送前确认、批准后可信自动续跑、唯一归属慢脑终局核验与终态展示全部通过。Windows原生对等证据继续作为Story级Deferred项，不由本样本外推。
