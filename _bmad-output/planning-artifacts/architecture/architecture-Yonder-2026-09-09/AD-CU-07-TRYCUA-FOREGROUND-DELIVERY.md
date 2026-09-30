# AD-CU-07 trycua 单次前台投递升级路线

- 状态：Accepted（macOS-only，Windows暂缓）
- Story：CU-S4
- OpenSpec：`cu-s4-trycua-foreground-delivery-spike`
- 日期：2026-09-30

## 问题

Accepted AD-E0-02固定`@trycua/cua-driver@0.25.0`。正式Yonder企业微信样本中，精确窗口后台快捷键、像素点击和文本输入均不能取得确认后置事实；继续使用`unverifiable`聚焦会产生敏感文本误投风险。

## 决定

2026-09-30统一fixture最初证明：0.25.0与0.30.4均可完成原子`type_text(x,y,text,foreground)`，因此一度保留0.25.0。随后正式Yonder QQ音乐产品样本补充了fixture未覆盖的受限快捷键场景：0.25.0的原生`HotkeyInput`不含foreground，而0.30.4工具Schema明确为hotkey公开精确target与`delivery_mode`。0.25.0后台`cmd+f`在真实窗口得到`observed + action_succeeded=false`；强制要求foreground时因能力缺失进入`observe-failed`，无法满足产品约束。

据此接受0.30.4作为唯一产品版本。它已经通过固定完整性、错误窗口拒绝、confirmed、SDK/原生双Observe、原前台恢复、shutdown后拒绝与进程清理；产品仍只开放封闭`cmd+f`、坐标点击和坐标文本等通过场景级验证的能力。若原前台身份在动作前失效，必须fail-closed。不得以桌面全局点击、自由键盘宏、AppleScript或第二Driver兜底。

## 迁移围栏

- 产品只保留固定0.30.4，不保留0.25.0兼容分支或长期双协议。
- SDK参数转换留在Adapter Worker；Application、Domain和Rust协议不引用具体SDK类型。
- `unverifiable`、`partial`、超时、断连和恢复失败均不得推进敏感文本或自动重试。
- Windows按用户决定暂缓；macOS结论只授权macOS候选范围。
