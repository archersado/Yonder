# AG-S5 Agent用户输入通道

Story: AG-S5  
Epic: AG  
Status: implementing  
OpenSpec: ag-s5-agent-input-channel

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前门禁

AD-VI-02已Accepted。协议1.14、Application输入用例、macOS Local Socket会话注册及VI-S1最终转写自动投递已实现。Codex CLI 0.156.1共享App Server薄桥接已实现并通过隔离合约验证；真实TUI会话的独立macOS验证仍待完成。云端输入须等待AG-S1先交付已认证WSS `AgentSession`；Windows产品接线按当前决定暂缓。

2026-09-18 Codex当前turn投递Goal失败：Codex CLI 0.154的`codex queue`只确认写入持久队列，活动turn没有收到steer，不能映射为`accepted`；`exec resume`会启动第二Agent，内部数据库轮询又违反通道边界。见[失败证据](../../../../openspec/changes/ag-s5-agent-input-channel/verification-codex-active-turn-macos.md)。AG-S5返回实施阶段，等待受支持的当前会话`turn/start|turn/steer`连接入口。

同日失败关闭修复完成：探针仅在前一turn结束后作为下一turn到达；CLI已移除`codex queue`调用，`yonder agent-bridge`在缺少可确认提交的当前会话通道时明确退出，不再让桌宠显示虚假连接或成功。

2026-09-23 复验共享 App Server 控制通道：当前 CLI 会话仍为`notLoaded`，`turn/start`返回`thread not found`，不能作为当前会话输入通道；AG-S5继续等待受支持的直接输入接口。

2026-09-24 Codex CLI 0.156.1已提供受支持的共享 App Server、`--remote`和`turn/steer`确认语义。实施转为显式共享会话模式：Codex TUI与Yonder薄桥接必须连接同一App Server，且用户显式绑定`thread_id`。桥接只直接调用`turn/steer|turn/start`，不再从持久队列推测成功。

关联实施目录：[ag-s5-agent-input-channel](../../../../openspec/changes/ag-s5-agent-input-channel/proposal.md)，路径`openspec/changes/ag-s5-agent-input-channel/`。
