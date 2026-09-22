# Proposal：CX-S2 圈选附件提交

关联Story：CX-S2；关联决策：Accepted AD-CX-01、AD-CX-02、AD-VI-02。

在现有`AgentSession`和`agent.input`链路增加显式附件能力协商、单附件有界分块与会话引用；macOS确认卡提供问题输入和“发送”，只有Agent确认accepted后显示成功并清理。Yonder不创建任务，Agent收到输入后自行调用既有`task.create`。

Architecture Impact：protocol-change。只扩展Rust协议唯一来源、Application输入用例、Desktop本地会话Adapter和现有确认卡；不新增连接、数据库、Outbox、截图文件或依赖方向。云端产品WSS仍受AG-S1门禁，本Change先以macOS本地受控Agent完成验证；Windows继续按用户决定暂缓。
