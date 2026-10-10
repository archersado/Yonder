# Proposal：AG-S10 圈选提交失败恢复与错误反馈

关联 Story：AG-S10。

## Why

圈选投递的任何非成功结果都会提前清空截图和问题，导致用户丢失输入；大截图超过 Agent 4 MiB 上限时还被折叠为泛化拒绝。

## What Changes

- 只有 Agent 明确 accepted 后清理提交临时数据。
- rejected、本地校验失败和 unknown 保留图片与问题，并区分能否重试。
- 在 AgentInputHub 投递前校验协议附件上限并返回稳定错误。
- 补 Application、Desktop 和 UI 回归测试及 macOS 独立验证。

## Impact

Architecture Impact：none。只修正既有临时会话终态与错误投影，不改变协议、Gateway、持久化或技术栈。
