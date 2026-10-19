# EX-S2 macOS 终局核验唤醒证据

日期：2026-10-19。入口为 `dev` 正式 debug `Yonda.app`、安装包内 `yonder mcp`、Agent Gateway、ExecutionRuntime、Sky CUA Worker 与 Codex Agent Bridge；未直连 SQLite、Socket 或 Driver。

- 企业微信七槽位敏感发送样本 `task_43de32dede4e13cadcc5083f03b4a21f` 在一次本机确认后自动执行发送动作，序号 29 为 `observed(action_succeeded=true, observe_valid=true)`，但旧 Bridge 把握手版本钉在 1.51，未声明协议 1.53 终局核验输入，任务停在 `running@31`。归属慢脑读取新鲜事实后于序号 32 核验达成、序号 33 完成；证据不记录收件人或正文。
- 修复后 Bridge 使用发布包当前 Rust 协议版本，并由握手合约锁定 1.53 以及既有输入、附件、replan、回复能力。Codex Bridge 18/18、协议终局输入门禁、Desktop 终局投递与浮窗等待用例全部通过。
- 无外部副作用正式回归 `task_6b80801d7fb243675082c16a02564fc5` 通过 Yonder 启动 Finder；序号 5 为有效 Observe，片段在序号 7 等待终局核验。Connector 重连后，Yonder 自动向唯一归属 Codex 会话投递 `goal_verification` 输入；慢脑重新读取 `task.get` 与 `task.events`，于序号 8 核验达成、序号 9 完成。该输入真实抵达当前活动会话，任务终态关闭顶部浮窗。

Windows 对等验证继续按用户决定暂缓，不由本证据外推。
