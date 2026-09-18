# ST-S3 架构设计

## 边界与依赖

Adapter复用现有rusqlite依赖，application→domain/protocol及adapter→application方向不变。按AD-ST-01增加显式open_unencrypted，不复制SQL实现或第二TaskStore。

## 状态与契约

SQLite当前表唯一事实源；现有schema v2、归属、sequence、事件/Outbox事务不变。使用中性SqliteTaskStore类型，历史SqlCipherTaskStore作为兼容别名；原open带密钥方法保留，未加密方法不设key、不探测自动回退。共享初始化先读取版本验证，再执行合法建库事务。

## 失败与验证

已有错误格式/加密库在初始化写入前拒绝；不自动覆盖/迁移。所有错误沿用StorageUnavailable，不输出文件正文。合约测试验证真实文件持久化、分页归属、Outbox失败回滚、恢复与加密文件字节不变。既有加密测试继续回归，不证明桌面接线完成。
