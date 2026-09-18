# Epic BU：Browser Use 集成

Epic: BU

## 模块边界

复用 ego-lite Task Space 和状态映射，不复制浏览器执行模型。

## Stories

- [BU-S1 ego-lite 集成与平台能力](story-BU-S1/README.md)
- [BU-S2 Task Space 引用与任务事实映射](story-BU-S2/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
