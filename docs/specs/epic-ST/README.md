# Epic ST：存储与数据安全

Epic: ST

## 模块边界

SQLCipher/FTS5/附件、事务和凭据；密钥相关工作暂停。

2026-09-14用户变更：MVP暂不加密，SQLite事务持久化继续实施；SQLCipher/附件加密、系统凭据与明文迁移延期MVP之后，归ST-S2，依据AD-ST-01。既有Spike证据保留。

## Stories

- [ST-S1 加密存储技术验证](story-ST-S1/README.md)
- [ST-S2 产品凭据与子密钥接线](story-ST-S2/README.md)
- [ST-S3 MVP未加密任务存储](story-ST-S3/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
