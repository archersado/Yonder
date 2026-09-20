# AD-VI-02 Agent用户输入通道

状态：Accepted  
日期：2026-09-17  
关联：AG-S5、VI-S1、CX-S2

## 决策

Yonder增加模型无关的`AgentInputSink`，把显式用户入口产生的最终输入主动交给交互开始时绑定的Agent会话。该消息不是任务，不写任务SQLite，不触发Application规划；Agent收到后自行决定是否创建任务。

`AgentInputSink`通过Agent Runtime已建立的双向连接发送`agent.input`，不依赖MCP。本地Agent Runtime与云端Agent Runtime使用同一会话、能力和确认语义；仅传输Adapter不同：本地使用UDS/Named Pipe，云端使用Yonder主动建立的单一WSS。云端Runtime直接在该WSS双工信道接收`agent.input`并返回accepted，不增加第二层Agent连接器。本地智能体若不能原生实现Yonder双工协议，才由其连接器把消息映射到当前会话入口。映射必须调用该Runtime的当前会话输入能力：有活动turn时steer，空闲或前一turn已中断时开始新turn；不得映射到Agent内部待处理队列。没有真实会话写入能力的本地连接器不得声明`user_input`。

Gateway按已认证连接注册`AgentSession`；每个会话对象直接持有其返回通道，Application只使用绑定的`session_id`投递，不按Agent厂商或进程名选择Adapter。任务内入口绑定任务归属Agent的当前会话；桌宠独立入口绑定显式激活的会话。仅有一个可接收输入的会话时可自动激活；多个会话且没有明确激活项时必须提示选择，不能发送到“最近连接”的猜测目标。交互开始后目标冻结，直到accepted、失败或取消。

连接注册表也是桌宠Agent连接提示的唯一瞬时事实源。Desktop Adapter在首个会话建立、最后会话断开时直接发布连接状态事件；桌宠初始化可读取一次快照，但不得轮询任务数据库。该提示与任务生命周期正交，不改变任务状态所有者或持久化模型。

连接必须显式声明`user_input`能力并提供可验证会话绑定。`accepted`表示Runtime已接受本次steer或已创建新turn，不得用“已写入Runtime内部队列”冒充成功。前一turn被用户中断只终止该turn，不关闭`AgentSession`；后续`agent.input`必须仍能独立提交。正文仅在有界内存中保留到accepted或失败关闭，日志与Outbox不得记录正文。每条输入使用稳定`input_id`；超时、断连或未知结果不自动重试。

语音入口的显式开始同时授权本轮最终非空转写自动投递，不再二次确认；最终结果前取消则不投递。支付、外部发信、删除等后续副作用仍由Agent/Yonder在对应动作边界确认。

## 淘汰项

- 不以`task.create`承载用户输入。
- 不启动`codex exec`或第二Agent进程代替当前会话。
- 不用`codex queue`等Agent内部待处理队列承载`agent.input`。
- 不依赖标准MCP工具调用假装服务器可向当前对话追加用户消息；Agent连接器必须提供并验证真实输入Adapter。
- 不在Desktop或Application中硬编码Codex、Claude Code等具体Agent的命令或SDK。

## 验证与证据修正

macOS UDS与云端WSS协议样本通过；Windows Named Pipe统一样本于GitHub Actions Windows Server 2022通过（run 35223461108）。这些结果支持模型无关协议与传输分工，不等于产品连接认证已完成。

2026-09-18撤回“Codex当前会话样本通过”：Codex CLI 0.154的`codex queue`只写入持久队列，活动turn没有收到steer；探针仅在前一turn结束后作为下一turn到达。产品CLI已删除该误报桥接。架构选择继续Accepted，但Codex连接器子范围返回实施阶段，只有受支持的当前会话`turn/start|turn/steer`入口才能重新验证。

云端WSS产品接线必须先由AG-S1定稿端点配置、连接认证、TLS校验、断线与重连边界，并建立唯一已认证`AgentSession`。AG-S5只能复用该会话发送`agent.input`；Spike中的临时自签证书和关闭校验不得进入产品。MVP暂缓存储加密不授权匿名WSS或明文持久化云端凭据。

2026-09-20附件补充：Accepted AD-CX-02允许在同一已认证`AgentSession`内先传输有界会话附件，再由`agent.input`引用。附件不使用本机路径、不建立第二连接、不进入日志、任务库或Outbox；accepted、拒绝、unknown、超时、断连和协议错误均清理。产品协议字段与接线须由CX-S2独立Change实现和验证。
