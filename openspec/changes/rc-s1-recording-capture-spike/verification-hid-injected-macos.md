# macOS IOHID 注入对照 Verification Goal

日期：2026-09-17  
结论：PASS（程序注入负样本）

Input Monitoring权限为granted。CUA REPL点击Yonda产生28个CGEvent，原始IOHID标记为0，20ms关联窗口内`hardware_correlated=0`，全部归类`external_unknown`，没有派生用户轨迹。证据未保存按键、坐标、截图、AX文本或完整事件Payload。

结构化证据：[result.json](../../../spikes/recording-capture/evidence/hid-injected-macos-20260917/result.json)

该结果只证明程序注入能被保守拒绝。仍需真实物理输入正样本证明IOHID关联可接受用户事件；正样本通过前路线不进入产品。
