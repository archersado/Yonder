# Epic CM：命令执行

Epic: CM

## 模块边界

结构化命令、进程树停止、超时和输出限额。

## Stories

- [CM-S1 结构化命令执行](story-CM-S1/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
