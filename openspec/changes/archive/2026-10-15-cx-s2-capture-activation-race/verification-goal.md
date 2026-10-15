# Verification Goal：CX-S2 截图隐藏激活竞态

状态：PASS（macOS 单显示器，2026-10-15）

## 目标

在 macOS 正式 Yonda bundle 中，从桌宠打开圈选层并拖出有效区域后，截图主动隐藏造成的前台应用恢复不得清除会话；确认卡必须显示本轮内存截图。确认卡可见时真实切换应用仍须清场。

## 自动验证

- Desktop 阶段判断测试：`Capturing` 不清场，`Selecting` 与 `Reviewing` 清场。
- Application `region_preview` 测试：截图、确认、提交失败保留与所有终止路径清场不回归。
- 原生错误分类：未授权请求失败返回 `permission-required`，授权后捕获空结果返回 `capture-failed`。

## 验证结果

- `YONDA_EXPECT_APP_SWITCH=1 swift apps/desktop/check-region-preview-macos.swift`：PASS。选择中切换真实 TextEdit 会清场；截图主动隐藏保留 `Capturing` 会话并显示图片确认卡；确认卡中再次切换会清场。
- `YONDA_EXPECT_CAPTURE_CANCEL=1 swift apps/desktop/check-region-preview-macos.swift`：PASS。有效框选显示内存图片，取消后会话清零，重新打开不存在旧预览。
- `cargo test --locked -p yonder-application region_preview`：5/5 PASS。
- `cargo test --locked -p yonder-desktop preview_title_tests::`：2/2 PASS。
- `python3 scripts/check_architecture.py` 与 `openspec validate cx-s2-capture-activation-race --strict`：PASS。

结构化证据位于 `apps/desktop/evidence/cx-s2-capture-activation-race-macos-20261015/result.json`。证据只记录布尔结论、bundle、提交和平台边界，不保存截图内容、问题正文或 Agent Payload。

验证期间旧全量脚本在不属于本 Change 的最终静置超时段被环境中的 `close-button` 清场提前结束；该次运行不计为通过。随后以两个分别绑定本 Change 验收条件的独立模式复验，均为 PASS。

## 平台边界

本 Goal 只覆盖 macOS 单显示器。Windows、多显示器、物理显示器断开和运行中撤权仍沿用 CX-S2 既有门禁。
