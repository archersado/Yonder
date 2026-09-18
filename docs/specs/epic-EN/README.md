# Epic EN：工程规范与交付

Epic: EN

## 模块边界

研发围栏、规划关联、CI、验证及发布证据。

## Stories

- [EN-S1 模块 Epic 与 Story 设计门禁](story-EN-S1/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
