# TM-S3 未开始任务取消

关联docs/specs/epic-TM/story-TM-S3/三份设计及Accepted AD-TM-04。Architecture Impact：architecture-change（Rust协议1.2控制入口，无迁移/依赖变化）。仅取消created，由Agent所属身份或可信LocalUser调用，不执行Driver停止，不开放人工创建。完整暂停/接管取消门禁保留。
