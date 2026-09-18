# AG-S5 架构设计

## 边界与依赖

Application定义`AgentInputSink`用例；Desktop只提交已完成的用户输入，Gateway Adapter绑定真实连接与Agent会话。Rust协议类型是消息、Schema和TypeScript唯一来源。

本地或云端Agent Runtime在可信双向连接上声明`user_input`能力并注册会话标识。Yonder以模型无关的`agent.input`请求，按开始交互时捕获的会话绑定投递`input_id/source/content/created_at/deadline`，Agent返回accepted或稳定拒绝。断连即失效，不回退到任意Agent，不启动新的Agent进程。

Gateway只维护一种逻辑连接：`AgentSession`。本地Adapter承载于UDS/Named Pipe，云端Adapter承载于Yonder主动建立的单一WSS；两者进入同一Application用例并使用同一Rust协议类型，不能分别定义消息或状态机。云端Agent Runtime直接消费WSS上的`agent.input`并在同一信道确认，不需要Yonder侧厂商连接器；本地智能体只有在不能原生接入该协议时才使用薄适配器。

云端WSS的端点配置、连接认证、TLS校验和重连生命周期属于AG-S1。AG-S5不创建第二条WSS，也不保存云端凭据；只有AG-S1交付已认证`AgentSession`后才注册`user_input` sink。断线中的输入返回unknown，不进入Outbox或自动重试。

每个`AgentSession`由可信连接注册`agent_id + session_id + connection sink`，并直接持有该连接的`AgentInputSink`，因此路由不需要识别Agent厂商或另选传输。任务内入口使用任务归属Agent的当前连接；桌宠独立入口使用显式激活连接；单一可用连接自动激活。多个可用连接且没有明确激活项时返回`target_required`，由界面让用户选择。语音开始时捕获连接绑定，收音期间连接变化不改投递目标。

## 状态与契约

Agent连接注册表同时是“是否至少存在一个可接收输入会话”的瞬时事实源。首个会话注册和最后会话移除时，Desktop Adapter直接向桌宠发布连接状态事件；桌宠启动时只读取一次当前快照，不轮询SQLite。连接状态以独立徽标叠加在任务生命周期动画之上，不覆盖`executing`、`waiting_for_user`等状态；连接变化会唤醒隐藏中的桌宠。

现有MCP工具调用是Agent→Yonder单向路径，不能冒充反向输入。Codex、Claude Code等连接器各自提供真实“向当前会话追加用户输入”的Adapter；具体CLI、SDK或会话API只存在于连接器内，不能进入Desktop、Application或通用协议。连接器应使用Runtime的统一输入入口：活动turn时steer，空闲或中断后开始新turn；Runtime返回已接受steer或新turn后才返回`accepted`。禁止用新Agent进程代替当前会话，禁止把文字伪装成`task.create`，也禁止用Runtime内部待处理队列承载该输入。

正文只驻留于有界内存直至ack或失败关闭。普通日志、SQLite任务事件和Outbox只允许记录`input_id`与结果码，不记录正文。交付超时结果未知时不得自动重试。

协议候选限制正文为非空UTF-8且不超过16 KiB，默认确认期限10秒、最大60秒。`source`首批支持`voice`与`selection`。连接器必须按`input_id`去重并对重复请求返回原结果；Yonder对`unknown`不自动重试。

## 失败与验证

断连、超时、拒绝和目标不明确返回稳定错误；不得回退到其他 Agent。验证覆盖本地双工连接、重连后的独立输入、正文不落盘和桌宠连接状态事件；云端 WSS 与 Windows 接线分别保留平台证据。
