# 任务

- [x] CX-S2 三份设计与 AD-CX-01 Preview 接线范围复核
- [x] 创建 macOS Preview OpenSpec
- [x] 实现显式入口、单显示器选择层与临时内存截图
- [x] 实现确认卡、Esc/取消/超时/权限失败清场
- [x] 修复确认卡尺寸，并实现框选/笔画选择与外接矩形截图
- [x] 独立 macOS Verification Goal

## 研发任务（按依赖顺序）

### [x] T1 收口 Preview 会话所有权与清理

- 目标：由Application唯一持有`idle/selecting/capturing/reviewing/cancelled/failed`及有界截图字节，并让所有结束路径清零。
- 模块：`crates/application`、`apps/desktop`组合根与现有macOS截图Adapter；不进入Domain或Protocol。
- 输入/输出：输入为显式开始、选区、捕获结果、重新圈选、Esc/取消/超时/关闭；输出为状态、稳定错误分类和仅供验证的无正文计数。
- 约束：复用现有能力；不新增依赖、协议、数据库、临时截图文件或单实现抽象；WebView不得保存权威会话状态。
- 验收：每条结束路径均回到`idle`且截图/笔画/选区字节为0；重新打开无旧预览；并发第二次开始被拒绝。
- 验证：Application最小状态机单元测试，加Desktop命令合约测试；测试不得输出截图内容。

### [x] T2 统一入口、拒绝与UI结束路径

- 目标：小龙和菜单栏使用同一Application开始用例；权限、租约与异常均可见且fail closed。
- 模块：`apps/desktop/src/main.rs`、`apps/desktop/ui/region-preview.*`及现有macOS截图接线。
- 输入/输出：输入为两类入口和Application结果；输出为单一选择层、确认卡或`permission-required`/`desktop-control-active`等稳定反馈。
- 约束：租约检查只读且发生在显示覆盖层前；不暂停/驱动CUA，不调用AG-S5、TaskStore、Outbox或Recording；不增加常驻监听。
- 验收：两入口行为一致；捕获前隐藏选择层；Esc、关闭、取消和30秒超时都先清理Application会话再隐藏窗口；入口错误不静默。
- 验证：扩展现有macOS原生脚本覆盖两入口、框选、画圈、重新圈选、物理Esc、取消和超时；AX/JSON只记录状态、尺寸和错误分类。

### [x] T3 补齐隔离验证夹具与边界快照

- 目标：在不保存截图正文的前提下复核权限拒绝、真实CUA租约拒绝、不持久化、不发送和无常驻指针。
- 模块：`apps/desktop/check-region-preview-macos.swift`及最少一个编排脚本；复用现有任务/Admission和临时bundle夹具。
- 输入/输出：输入为无敏感四色窗口、临时bundle身份和真实Desktop租约；输出为单个结构化`result.json`及前后计数/文件清单差异。
- 约束：不执行`tccutil reset`，不读取数据库正文，不保存截图/base64/窗口树，不引入新依赖；清理只作用于本次临时bundle和测试任务。
- 验收：验证矩阵每行均有布尔判据；任务/事件/Outbox/索引计数不变；无图片文件、子进程或指针监听残留。
- 验证：先运行脚本自检与自动回归，再由独立验证者在真实macOS上执行`verification-macos.md`矩阵。

### [x] T4 独立 Verification Goal 与归档门禁

- 目标：由非实现者复跑同一提交并判定Preview是否通过。
- 模块：`openspec/changes/cx-s2-macos-preview/verification-macos.md`与证据目录。
- 输入/输出：输入为T1～T3通过的提交、环境清单和验证命令；输出为不含敏感正文的结构化证据及通过/失败结论。
- 约束：验证者不修改实现；任一失败返回实施阶段；只有全矩阵通过才能Archive。
- 验收：证据可对应CX2-01/02/04/05/06/07的Preview部分，且明确不代表完整CX-S2或Windows通过。
- 验证：复核提交/二进制哈希、矩阵结果、证据字段和数据清理；通过后更新状态与任务勾选。

## 延期任务

- [x] Agent提交、问题输入与VI-S1语音组合：已移出本 Preview，留待后续 Change
- [x] CUA任务暂停与恢复语义：已移出本 Preview，留待后续 Change
- [x] 多显示器、负坐标、显示器变化与运行中撤权：已移出本 Preview，留待后续 Change
- [x] Windows实现与双平台统一样本：已移出本 Preview，留待后续 Change
