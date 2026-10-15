# Verification Goal：CX-S2 截图隐藏激活竞态

状态：待验证

## 目标

在 macOS 正式 Yonda bundle 中，从桌宠打开圈选层并拖出有效区域后，截图主动隐藏造成的前台应用恢复不得清除会话；确认卡必须显示本轮内存截图。确认卡可见时真实切换应用仍须清场。

## 自动验证

- Desktop 阶段判断测试：`Capturing` 不清场，`Selecting` 与 `Reviewing` 清场。
- Application `region_preview` 测试：截图、确认、提交失败保留与所有终止路径清场不回归。

## 原生样本

待用当前提交构建、临时签名的正式 `Yonda.app` 运行。证据只记录布尔结论、窗口标题与阶段，不保存截图内容、问题正文或 Agent Payload。

## 平台边界

本 Goal 只覆盖 macOS 单显示器。Windows、多显示器、物理显示器断开和运行中撤权仍沿用 CX-S2 既有门禁。
