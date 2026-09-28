# TM-S8 统一产品执行闭环

Story: TM-S8  
Epic: TM  
Status: verifying  
OpenSpec: tm-s8-unified-product-chain

## 目标

用真实 Codex MCP 慢脑连接正在运行的 Yonder，分别完成 CUA、BUA、Office 与 Command 的产品级任务闭环。不得用测试宿主、直接 Adapter、私有脚本逐步调用或手工改库替代产品入口。

## 文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前结论

四类底层能力及统一启动已有 macOS 子范围证据，但尚无一份同时证明真实慢脑 MCP 接入、Yonder GUI 可见、完整编排、确认、执行、Observe、审计与终态的统一产品证据。本 Story 专门关闭该差距；Windows 按用户决定暂缓，不据此标记双平台 Done。
