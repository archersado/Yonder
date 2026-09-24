# AG-S5 Codex Remote TUI macOS Verification Goal

日期：2026-09-24  
结论：PASS（Codex本地产品桥接）

## 目标

使用真实Codex CLI remote TUI和共享App Server，独立验证Yonder输入进入同一thread：活动turn使用steer，中断后的下一条输入开始新turn，且只有App Server确认后才返回accepted。

## 环境与方法

- 平台：macOS；Codex CLI 0.156.1。
- 启动`codex app-server --listen unix://<temporary-socket>`，再以`codex --remote unix://<temporary-socket>`创建真实TUI会话。
- Yonder桥接以显式socket和thread标识连接同一个WebSocket-over-Unix-Socket App Server；临时Yonder Gateway只发送协议输入并记录结构化回执，不记录正文。
- 第一阶段在TUI turn执行期间投递；第二阶段由操作员在TUI中按Esc，使当前turn进入`interrupted`，再投递下一条输入。

## 结果

- `thread/loaded/list`确认显式thread已由该App Server加载，桥接随后才向Yonder声明`session_id`和`user_input`能力。
- 活动turn阶段收到`accepted=true`，真实TUI在同一thread继续处理输入，证明`turn/steer`路径可用。
- TUI明确显示`Conversation interrupted`后回到空闲态；下一次投递再次收到`accepted=true`，真实TUI显示新的用户turn并生成对应响应，证明中断后`turn/start`路径可用。
- Yonder Gateway断开后桥接失败关闭；停止TUI和App Server后，按临时socket路径及测试桥接命令检查无残留进程。
- 隔离合约脚本另行验证RPC方法序列、通知排空和匹配ID确认；本Goal不依赖内部数据库、持久队列、第二Agent或自动会话猜测。

## 范围结论

Codex本地产品桥接的macOS独立门禁通过。该结论不覆盖云端WSS接线，也不替代Windows产品验证；后者按用户决定暂缓，因此AG-S5保持`implementing`且不得Archive。
