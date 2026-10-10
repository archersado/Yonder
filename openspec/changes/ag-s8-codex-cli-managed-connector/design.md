# 设计

## 接线

`SessionStart(startup|resume)` 后台 Hook 把事件 JSON 写入 `yonder codex-hook` stdin。CLI 只接受当前事件的 `session_id`，幂等启动共享 daemon，并把 `codex app-server proxy` 的 stdin/stdout 组合成双向字节流，在其上完成 WebSocket Upgrade，再发送 initialize、initialized、thread/resume 和 start/steer RPC。随后复用协议 1.33 的 Agent Gateway hello 与 agent.input/replan-input。不得把 proxy 当作 JSONL，也不得另起会与当前 writer 冲突的独立 stdio App Server。

Hook 默认使用正式 MCP 配置的 `codex-cli` Agent 身份；`YONDER_AGENT_ID` 只作为显式覆盖。不得因 Hook 进程未继承 MCP 专属环境变量而退化成另一个未登记身份。

## 恢复

连接器在进程内保留有界输入 ID 去重集合。Yonder UDS 断开后，退避封顶且总恢复窗口有限，只重新发送 hello，不重放输入。Codex RPC 在投递中断时把该 input ID 记为拒绝/未知终态；proxy 断开则连接器失败关闭，由下一次显式 Hook 生命周期恢复。

## 分发

仓库提供可审阅的 Codex Hook 配置；安装与信任保持显式。Hook 命令只依赖 PATH 中安装好的 `yonder`，不包含本机绝对路径。产品文档列出兼容版本与诊断命令。
