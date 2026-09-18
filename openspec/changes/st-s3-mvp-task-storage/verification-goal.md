# ST-S3 独立 Verification Goal

日期：2026-09-14。Story：ST-S3；Change：st-s3-mvp-task-storage；AD：AD-ST-01。状态：本机存储核心范围PASS；Windows、PR及完整产品未完成，不Archive。

## 验证目标与证据

实现完成后独立检查真实文件与Application用例，不修改产品实现。运行`cargo test --offline --locked -p yonder-adapters`：10 passed，0 failed。新增合约测试位于crates/adapters/src/task_store.rs，名称`unencrypted_storage_persists_recovers_rolls_back_and_preserves_rejected_files`，使用独立临时目录，成功后仅清理本次生成文件。

| AC | 实际观察 | 判定 |
|---|---|---|
| AC1 | 无密钥创建/重开SQLite，文件头SQLite format 3，任务持久化 | PASS |
| AC2 | 人为Outbox插入失败后状态仍running/sequence=2，事件和Outbox均为2；解除测试触发器后恢复提交一致 | PASS |
| AC3 | 两个不同Agent任务经本机查询跨页可达，Agent查询仅自身；重开后恢复2个running，重复恢复0，状态interrupted、事件/Outbox为3 | PASS |
| AC4 | 未知schema、错误文件、加密旧库均拒绝；拒绝前后文件字节相同，加密库仍能用原密钥查询 | PASS |

现有9项加密、分页、权限、恢复、准入及收起协调测试一并通过。架构依赖/规划关联检查与git diff --check通过。ts-rs仍有原有deny_unknown_fields解析提示，未因此更改协议或生成物。

## 范围与限制

未加依赖、表结构、迁移、Gateway或密钥实现。当前依赖构建仍包含既有SQLCipher库，未加密入口不设key，标准SQLite格式已验证；本次不为去除历史依赖改动锁文件。SqlCipherTaskStore兼容别名仍共用SqliteTaskStore，不维护两套SQL逻辑。

此项只证明存储基础能力；DS-S2界面、可信桌面身份、单实例恢复及真实执行器尚未接线。Windows验证继续按用户要求暂缓，无本轮PR，禁止标记完整Story Done或Archive。
