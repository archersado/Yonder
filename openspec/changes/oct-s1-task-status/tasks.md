当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 任务

## 已实施的历史范围

- [x] 记录多任务需求及 AD-OCT-03 Proposed 决策，同步 Epic/Story/OpenSpec
- [x] task.list 进程内契约、Rust 派生协议、SQLCipher 游标分页及 macOS 核心验证（verification-task-list.md）
- [x] AD-OCT-04 归属字段与可信 AuthContext 查询隔离（verification-task-ownership.md）
- [x] AD-OCT-05 gateway.hello、版本协商及连接内查询门禁（verification-gateway-hello.md）
- [x] AD-OCT-06 进程内资源准入及多线程竞争验证（verification-resource-admission.md）
- [x] 可信启动用例串联资源准入和状态事务（verification-admitted-start.md）
- [x] 确认停止后的终态事务与资源释放（verification-admitted-finish.md）
- [x] 正式 Workspace、Domain 状态转换、Application CAS/恢复用例、SQLite 事务与 Rust 只读协议历史批次
- [x] CI 依赖方向、协议漂移和 Story/OpenSpec/PR 关联的本地门禁（verification-ci.md）

## 后续范围迁出

- [x] 生产本地/云端认证、AuthContext、Gateway、CLI/MCP与双平台传输从本Change撤出，由AG-S1及关联Change承接；不表示已完成
- [x] task.list全量查询与忙碌派生从本Change撤出，由TM-S6承接；不表示完整TM-S1已完成
- [x] 任务总览、分页、详情、桌宠状态与ego-lite关联从本Change撤出，由DS-S2、BU-S2与TM-S5承接
- [x] 真实执行attempt、Driver派发/Observe、步骤声明与停止确认从本Change撤出，由TM-S2/TM-S3、CU-S2与AG-S3承接
- [x] 文件身份、同文件互斥与安全写入从本Change撤出，由FI-S1承接
- [x] 桌面组合根、单实例、恢复和原生UI从本Change撤出，由DS-S1/DS-S2承接
- [x] Credential Store、加密与密钥接线从本Change撤出，依AD-ST-01延期至MVP之后ST-S2；不得从本Change恢复实施
- [x] Windows/macOS CI、原生E2E、PR隔离及Archive从本混合Change撤出，由EN-S1和各产品Story独立验证

本Change冻结且不Archive。上述勾选表示“迁出动作完成”，不表示目标Story的产品验收完成。
