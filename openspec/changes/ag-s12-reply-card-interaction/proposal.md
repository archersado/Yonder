# Proposal：AG-S12 圈选回复卡呈现与拖拽

关联 Story：AG-S12。

## 背景

实际使用显示，AG-S11 的受限 Markdown 在松散列表与宿主默认 marker 上仍会呈现得像普通换行文本；无边框圈选卡也无法由用户移开。

## 变更

- 修复常用列表的语义归并和显式视觉标记。
- 在卡片标题区接入最小权限的原生窗口拖动。
- 增加 UI 合约与 macOS 正式宿主验证。

## Architecture Impact

`none/conforming`。状态所有者、依赖方向、协议、持久化与技术栈均不变；只使用既有 Tauri 窗口能力和本地安全 DOM 渲染。
