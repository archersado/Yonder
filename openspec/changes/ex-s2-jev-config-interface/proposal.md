# Proposal：EX-S2 最小 Jev 配置界面

关联 Story：EX-S2；关联 ADR：AD-EX-03。

实现 Application 拥有的最小 Jev 配置模型、SQLite 单行持久化和本机 Task Space 配置区。本 Change 只交付配置读取、校验和保存，不接 Jev 执行、不调用模型、不改变任务状态或事件/Outbox。执行接线仍受 AD-EX-02 双平台门禁约束。
