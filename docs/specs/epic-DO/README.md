# Epic DO：文档处理

Epic: DO

## 模块边界

Document Port 与 OOXML 保真读写、文件锁和另存。

## Stories

- [DO-S1 OOXML Adapter 选型](story-DO-S1/README.md)
- [DO-S2 OOXML Document Port](story-DO-S2/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
