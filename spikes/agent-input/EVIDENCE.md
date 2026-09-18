# AG-S5 Agent Input Spike 证据

日期：2026-09-17  
状态：macOS UDS、Windows Named Pipe与云端WSS样本通过；AD-VI-02已Accepted。

## 通用双向会话

运行`python3 spikes/agent-input/duplex_probe.py`。Agent在UDS持久连接声明`user_input`和`session_id`，Yonder在同一连接主动发送`agent.input`，Agent返回同`input_id`的`accepted`。拒绝稳定返回`rejected`；断连和超时归类为`unknown`且不自动重试；重复`input_id`只交付一次。探针不使用MCP、不持久化正文。

据此保留64 KiB帧上限，并将正文候选限制为16 KiB、默认确认期限10秒、最大60秒；该限制足以容纳语音与圈选文本，同时为协议元数据和错误响应保留空间。

## Codex连接器候选

在当前Codex会话运行`codex queue --thread "$CODEX_THREAD_ID" --message <探针>`，CLI返回已排队消息与同一线程标识；该探针随后作为用户输入回到原会话。结论仅证明Codex连接器存在当前会话写入候选，不把`codex queue`提升为Yonder通用协议或核心依赖。

## 云端Runtime WSS

运行`node spikes/agent-input/wss_probe.mjs`。Yonder样本主动建立WSS，发送`gateway.hello`与`agent.input`；云端Runtime样本在同一双工信道返回accepted。探针只使用Node标准库和临时自签证书，不增加产品依赖；临时连接为测试关闭证书校验，不能进入产品配置。

## 尚未通过

- 产品Rust协议与真实Runtime Adapter尚未实施。

## Windows Named Pipe

统一样本`windows_named_pipe_probe.ps1`在GitHub Actions `windows-2022`执行成功：run 35223461108，任务`named-pipe`及步骤“验证双向Agent输入”均为success。Windows产品接线按用户决定暂缓。
