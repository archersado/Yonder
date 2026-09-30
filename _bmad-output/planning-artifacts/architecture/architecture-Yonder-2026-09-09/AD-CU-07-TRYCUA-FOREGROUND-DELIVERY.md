# AD-CU-07 trycua 单次前台投递升级路线

- 状态：Accepted（macOS-only，Windows暂缓）
- Story：CU-S4
- OpenSpec：`cu-s4-trycua-foreground-delivery-spike`
- 日期：2026-09-30

## 问题

Accepted AD-E0-02固定`@trycua/cua-driver@0.25.0`。正式Yonder企业微信样本中，精确窗口后台快捷键、像素点击和文本输入均不能取得确认后置事实；继续使用`unverifiable`聚焦会产生敏感文本误投风险。

## 决定

2026-09-30统一样本证明：0.25.0与0.30.4的`listToolsJson/callTool`兼容层都公开精确窗口、窗口坐标与`delivery_mode`，两者对原子`type_text(x,y,text,foreground)`均完成错误窗口拒绝、confirmed、SDK/原生双Observe、原前台恢复、shutdown后拒绝与进程清理。0.30.4没有为当前阻断提供独占能力，因此淘汰升级，继续固定0.25.0。

接受macOS-only产品Apply前置：可在独立Change中把0.25.0现有的前台坐标文本能力接入封闭CUA语义。若原前台身份在动作前失效，样本拒绝动作且不修改目标；产品同样必须fail-closed。不得以桌面全局点击、自由键盘宏、AppleScript或第二Driver兜底。

## 迁移围栏

- 产品只保留0.25.0，不引入0.30.4或长期双协议。
- SDK参数转换留在Adapter Worker；Application、Domain和Rust协议不引用具体SDK类型。
- `unverifiable`、`partial`、超时、断连和恢复失败均不得推进敏感文本或自动重试。
- Windows按用户决定暂缓；macOS结论只授权macOS候选范围。
