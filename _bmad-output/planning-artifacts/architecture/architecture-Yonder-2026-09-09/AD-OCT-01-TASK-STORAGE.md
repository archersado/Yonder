# AD-OCT-01 任务事务存储

2026-09-14由Accepted AD-ST-01覆盖当前MVP加密门禁：任务存储允许显式未加密SQLite路径，不依赖Key Provider；下述SQLCipher决定保留历史。表结构、事务、恢复与版本拒绝规则不变，不自动解密或迁移旧库。

未定案建议见 AD-TM-01-TASK-METADATA.md（Proposed）：元数据、事件和版本方案须先对齐既有产品需求；不是当前决定。当前数据库仍为 v2，禁止自动迁移旧库。

状态：Accepted（OCT-S1 存储实现范围）。依赖 AD-E0-06。

2026-09-11 当前格式更新：依 AD-OCT-04，新库改为 schema_version=2，增加不可变任务归属。v1 保留但拒绝打开，迁移必须另建 Change；下述 v1 为初批历史决定，不再是新建库格式。

任务数据库初版 schema_version 为 1。tasks 保存当前状态和序号；events 保存前后状态；outbox 保存对应事件外键及待投递状态。三者同事务写入，Outbox 不复制第二份事件正文。创建产生序号 1 的 created 事件；迁移通过 expected_sequence 和前状态共同比较更新。

SQLCipher Adapter 只接受组合根提供的 32 字节数据库子密钥，使用 sqlite3_key 设置二进制密钥并校验 cipher_version；不读取其他 Adapter，也不保存密钥。无 Key Provider 时不得启动生产宿主。未来版本迁移另建 Change，当前遇到未知 schema_version 直接拒绝，禁止覆盖已有数据。

序号存储使用 SQLite 有符号整数，范围为 1 到 i64::MAX。事件分页最多 100。此实现不自动重试写冲突、不执行桌面动作、不暴露外部传输协议。

## 启动恢复补充（2026-09-11）

Application 通过 TaskStore 有界读取 running 任务，每批最多 100 条，复用 Domain Interrupt 和逐任务 compare-and-commit。每个任务的状态、事件、Outbox 原子更新；批次不要求全有或全无。失败立即返回错误，已提交任务保持 interrupted，未提交任务仍为 running；后续显式启动恢复只处理剩余 running，不重复追加已恢复任务的事件。

恢复是组合根在获得宿主单实例所有权后、开放 Gateway/执行器之前的显式启动步骤，不在数据库 open 时自动运行。宿主须重复调用至返回 0，出错时不得开放任务执行。单实例所有权和生产启动接线尚待桌面宿主实施，本次用例不提供这些保证。无新表、迁移或外部协议，不推断 unknown 动作的结果。
