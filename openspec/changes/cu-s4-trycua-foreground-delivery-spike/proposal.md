# Proposal：trycua 单次前台投递升级 Spike

关联Story CU-S4与Proposed AD-CU-07。Architecture Impact：none（隔离Spike，不修改产品依赖、协议、持久化或运行时）。

## Why

trycua 0.25.0在正式企业微信样本中无法确认后台聚焦和输入；产品必须先证明新版动作级前台投递能建立焦点、完成Observe并恢复用户原工作窗口，才能考虑升级。

## What Changes

- 固定比较0.25.0与0.30.4工具契约。
- 建立无敏感内容的macOS隔离fixture与统一动作样本。
- 验证精确窗口前台click、同窗口输入、双重Observe、恢复、失败注入和清理。
- 形成独立Verification Goal与AD-CU-07接受/缩小/淘汰结论。

## Non-goals

不接产品Gateway，不升级`apps/desktop/cua`，不发送消息，不验证Windows，不引入第二生产Driver。
