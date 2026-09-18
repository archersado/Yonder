# Epic FI：文件操作

Epic: FI

## 模块边界

规范路径与文件身份、受控读写、回收站及原子替换。

## Stories

- [FI-S1 规范文件身份与受控操作](story-FI-S1/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
