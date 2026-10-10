# Proposal：AG-S11 Agent 回复 Markdown 渲染

关联 Story：AG-S11。

## Why

Yonder 已能在圈选卡接收流式 Agent 回复，但 Markdown 仍作为原始符号显示，降低长回答的可读性。

## What Changes

- 在圈选回复卡增加流式兼容的受限 Markdown DOM 渲染。
- 支持常用块级与行内语法，并为代码、列表、引用补充样式。
- 禁止原始 HTML、脚本、外部图片加载和危险 URL；解析失败回退纯文本。
- 补 UI 合约、渲染单元与 macOS 正式 bundle 验证。

## Impact

Architecture Impact：none。只改变 WebView 的有界展示投影，不修改协议、Gateway、状态所有权、持久化或技术栈。
