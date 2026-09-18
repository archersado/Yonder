# Epic AG：Agent Gateway 与接入协议

Epic: AG

## 模块边界

本地 IPC、CLI/MCP、云端客户端接入和 Rust 派生协议；不实现云端服务。

## Stories

- [AG-S1 统一任务读取与 Gateway 握手](story-AG-S1/README.md)
- [AG-S2 Agent任务创建接入](story-AG-S2/README.md)
- [AG-S3 Agent步骤声明接入](story-AG-S3/README.md)
- [AG-S4 Yonder Agent Skill](story-AG-S4/README.md)
- [AG-S5 Agent用户输入通道](story-AG-S5/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。

- AG-S2仅允许Agent经Gateway创建，字段、认证和幂等按其Story推进。
- AG-S1当前增量实现安装包内`yonder` CLI、MCP stdio与macOS UDS生产接入，复用AG-S2/AG-S3能力。
- AG-S4当前只完成规格；等待BUA、CUA、Document、Command正式能力全部通过后再生成OpenSpec和Skill包。
