# AG-S5 Codex App Server桥接合约验证

日期：2026-09-24
结论：PASS（隔离合约层）

## 目标

在不访问真实Codex会话、不发起模型调用的隔离环境中，验证`yonder agent-bridge`对共享App Server的路由、确认和会话绑定语义。

## 验证方法

- 构建`yonder-cli`，用临时HOME和临时Unix Socket启动受控Yonder Gateway与WebSocket-over-UDS App Server双端。
- App Server首次返回`active + turn-active`，第二次返回`idle + interrupted`。
- Gateway连续发送两条`agent.input`，验证握手声明的thread、方法顺序和accepted结果。
- 命令：`python3 apps/yonder-cli/check-codex-agent-bridge-macos.py`。

## 结果

- 握手只声明`user_input`，`session_id`等于显式绑定的thread，不声明附件能力。
- App Server方法序列为`initialize → thread/loaded/list → thread/read → turn/steer → thread/read → turn/start`。
- 每个RPC回执前插入状态通知，桥接持续排空通知并仅以匹配ID的RPC结果判定accepted。
- 两条输入均在匹配的steer/start RPC确认后返回`accepted=true`。
- Gateway EOF后桥接非零退出，不保留虚假连接。
- `cargo test -p yonder-cli`的5项测试全部通过，含活动turn选择与1024条有界去重。

## 剩余门禁

本Goal只证明桥接合约和失败关闭，不替代真实Codex remote TUI会话证据。AG-S5仍需独立验证真实活动turn收到steer，且中断后同一thread的下一条输入成功开始新turn；Windows按用户当前决定暂缓，云端WSS继续受AG-S1门禁约束。
