# AG-S1 正式宿主查询独立 Verification Goal

2026-09-14；Story AG-S1 AC1–5；AD-OCT-05/AD-DS-01/AD-ST-01。状态：首批库接线PASS，传输认证/双平台/完整Story未通过，不Archive。

Application新增GatewaySession.handle_encoded复用唯一Rust协议encode；TaskHost.query_session接受可信会话并读唯一实际SQLite，不开放GUI/网络入口、不借用LocalUser身份。实际文件样本启动恢复后，两Agent新会话各自未握手查询-32002；hello后各仅读所属任务；越权get-32004；同身份新会话仍未握手。用户本机查询仍显示全部，既有任务状态映射回归保持。

cargo test --offline --locked -p yonder-desktop --lib -p yonder-application：Application6、Desktop3共9项通过。无新增依赖、协议schema或SQLite迁移，无新增UI/原生行为。传输认证、IPC、Agent任务创建尚未实现，不把Rust构造可信上下文的测试当真实认证证据。Windows仍暂缓。

新增Story文档门禁最初发现缺少标准章节及Proposal双向路径，已补齐文档关联；实现测试仍为9项通过，不把文档门禁失败改写为测试通过。
