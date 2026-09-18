# AG-S2 创建权限门禁独立 Verification Goal

2026-09-14；AG-S2 AC1/AC2/AC7；AD-AG-01。状态：核心权限子范围PASS；真实Gateway任务创建与完整Story未完成，不Archive。

Application共享create先验证Agent上下文，LocalUser返回PermissionDenied且不调用TaskStore。query错误映射沿用-32003。没有新建按钮、Tauri创建命令、HTTP或伪造Agent身份旁路；现有Agent内部create仍需调用方已认证，不把Rust类型检查当真实认证。

实际SQLite文件样本中，LocalUser("desktop")用合法ID manual调用create返回PermissionDenied，随后的get返回NotFound；然后两个Agent合法创建并恢复/查询成功，本机用户仍看全部，GatewaySession各Agent只读所属/越权拒绝/重连握手不变。Adapter现有事务、Outbox失败回滚、恢复、明文/加密入口与资源准入回归保持。

cargo test --offline --locked -p yonder-desktop --lib -p yonder-application -p yonder-adapters：Desktop3、Application6、Adapter10共19项通过。架构关联与依赖检查通过。无协议schema、持久化迁移、新依赖或UI/Driver行为变化；没有新原生录制或用户记录上传。

完整task.create方法、幂等事务、连接认证/传输尚待技术决策；接管记录与交回Observe按AD-TM-03继续TM/RC联合设计。Windows暂缓仍保留证据要求。本子范围不表示真实Agent已经能创建任务，更不表示Recording已开始。
