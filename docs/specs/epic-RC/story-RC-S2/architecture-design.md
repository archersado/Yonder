# RC-S2 架构设计

## 边界与依赖

Application拥有Trajectory派生、版本、校验和回放编排；Domain只定义无技术依赖的动作与状态规则。Recording读取、Trajectory存储、CUA执行分别通过Port访问，Adapter不得互调。Desktop只展示快照与提交用户意图。

回放不是独立执行栈：Application把已审阅动作逐步交给既有CU用例，沿用trycua SDK Worker、TM attempt、唯一前台桌面租约和动作后Observe。参考插件的`rpatap --play`不进入产品。

## 状态与契约

### Trajectory候选模型

每个版本至少包含`trajectory_id`、`version`、`source_recording_id`、`previous_version`、名称、完整性、动作序列和创建时间。每个动作包含稳定`action_id`、应用/窗口匹配、动作类型、AX目标、窗口内相对坐标回退、参数引用、预期结果与Observe策略。

原始Recording只追加不修改；Trajectory编辑采用复制生成新版本。协议、SQLite表、附件引用和删除传播属于架构变化，必须先建Architecture Decision，再从Rust类型生成协议。

## 派生规则

可以借鉴参考实现的确定性折叠思路：鼠标按下/抬起组成点击或拖动，短时间同方向滚动合并，连续普通输入形成一个文本动作，快捷键保持独立。具体时间、距离与步骤上限由Windows/macOS统一样本校准，不照搬插件常量。

动作目标优先来自AX/UIA语义身份，窗口内相对坐标只作显式回退。无法建立稳定目标、存在来源缺口或命中隐私排除时产生不可回放诊断，不生成看似有效的坐标动作。

## 回放状态与停止

候选状态为`draft → ready → requested → countdown → running → paused|completed|failed|cancelled|unknown`。`requested`后必须等待Agent真实创建任务和资源准入；UI不能自行进入running。

每步执行前解析新鲜目标，执行后Observe预期条件，再由Agent决定下一步。用户输入、暂停/取消、目标失效、Observe失败或Worker断连使用既有TM/CU控制事务；副作用结论不明进入`unknown`，不自动重试。

## 失败与验证

### 数据与安全

原始按键正文、截图和AX文本属于敏感内容，不进入日志或普通任务事件。MVP未加密背景下，字面文本持久化和截图资产保存保持阻塞；首批可使用参数占位和不含正文的结构验证。删除派生版本使用FI安全删除，并保留必要审计引用。

### 双平台门禁

Windows使用UI Automation和已选CUA路线，macOS使用Accessibility和同一CUA路线。统一样本必须覆盖不同窗口位置/DPI、目标重开、内容变化、输入暂停、步骤失败、崩溃恢复及`replay`来源不重录。RC-S1来源门禁和两平台证据未通过前不生成产品Proposal。
