# Epic RC：手动录制与回放

Epic: RC

## 模块边界

手动开始、不可变时间线、轨迹审阅与确认回放。

## Stories

- [RC-S1 手动录制与不可变时间线](story-RC-S1/README.md)
- [RC-S2 操作示教与安全回放](story-RC-S2/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
