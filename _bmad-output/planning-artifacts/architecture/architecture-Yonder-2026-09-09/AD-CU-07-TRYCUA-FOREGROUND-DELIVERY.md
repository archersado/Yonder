# AD-CU-07 trycua 单次前台投递升级路线

- 状态：Proposed
- Story：CU-S4
- OpenSpec：`cu-s4-trycua-foreground-delivery-spike`
- 日期：2026-09-30

## 问题

Accepted AD-E0-02固定`@trycua/cua-driver@0.25.0`。正式Yonder企业微信样本中，精确窗口后台快捷键、像素点击和文本输入均不能取得确认后置事实；继续使用`unverifiable`聚焦会产生敏感文本误投风险。

## 候选决定

在2026-10-02前限时验证固定候选0.30.4的动作级前台click。候选只有在精确窗口、焦点Observe、同窗口文本Observe、原前台恢复、错误目标拒绝和关闭清理全部通过时，才可进入独立产品Apply Change。

Spike不修改AD-E0-02当前Accepted决定。未通过、只部分通过或期限届满时，产品继续固定0.25.0并保持相关能力fail-closed；不得以桌面全局点击、自由键盘宏、AppleScript或第二Driver兜底。

## 迁移围栏

- 产品只保留一个trycua版本，不长期维护0.25/0.30双协议。
- SDK参数转换留在Adapter Worker；Application、Domain和Rust协议不引用具体SDK类型。
- `unverifiable`、`partial`、超时、断连和恢复失败均不得推进敏感文本或自动重试。
- Windows按用户决定暂缓；macOS结论只授权macOS候选范围。
