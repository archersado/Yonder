# AG-S5 Codex当前Turn投递 Verification Goal

日期：2026-09-18  
结论：FAIL

## 目标

验证Yonder的Codex薄桥接把一条不改变当前研发目标的输入提交到当前会话：活动turn必须进入steer；空闲或中断后的会话必须开始新turn。只有Runtime接管输入后才能返回`accepted`，CLI内部排队成功不算通过。

## 结果

- 本机Codex CLI版本为`0.154.0`，App Server生成协议包含`turn/start`、`turn/steer`和`turn/interrupt`。
- 当前ChatGPT/Codex会话只暴露`CODEX_THREAD_ID`及产品IPC；没有可供Yonder复用的App Server控制socket。
- 活动turn期间执行现有`codex queue --thread ... --message ...`后，CLI仅返回`Queued message`，当前turn没有收到steer。
- 不读取正文检查队列表时，该线程仍有一条待处理记录，证明CLI成功不等于Runtime已接管输入。
- 前一turn结束后，探针才作为下一turn输入进入同一会话，随后待处理记录清零；这证明空闲后投递可达，但不满足活动turn的steer语义。
- `codex exec resume`会启动第二个Agent进程，不是同一会话输入通道，按AG-S5边界淘汰。
- 产品CLI已删除`codex queue`桥接并改为失败关闭；`yonder agent-bridge`以非零状态和稳定原因退出，不再注册虚假的`user_input`能力。

结构化证据：`apps/yonder-cli/evidence/agent-input-codex-active-turn-20260918/result.json`。证据不记录输入正文。

## 结论与门禁

现有`agent-bridge`已停止把`codex queue`退出成功映射为`accepted`。在Codex为当前会话提供可复用的`turn/start|turn/steer`SDK、CLI或受支持App Server连接以前，不得用读取内部队列数据库、修改Codex机制或启动第二Agent绕过。本子目标返回实施阶段，AG-S5保持implementing。
