# macOS IOHID 用户正样本 Verification Goal

日期：2026-09-17  
结论：FAIL（IOHID时间关联路线淘汰）

## 目标

在Input Monitoring已授权时，以20ms和100ms窗口将真实物理输入的IOHID标记与CGEvent关联；只输出计数和最小时间差，不保存输入正文、usage、坐标、截图或AX文本。

## 结果

三轮真实物理输入样本中，首轮得到3个CGEvent和1个IOHID标记但关联数为0；后两轮100ms校准仍无法稳定获得IOHID标记或相关事件。结果不能稳定证明`user`来源，按AD-RC-01淘汰门槛失败。结构化证据见[结果](../../../spikes/recording-capture/evidence/hid-user-macos-20260917/result.json)。

程序注入负样本可保守拒绝并不足以弥补真实用户正样本失败。该路线不得接产品；RC-S1保持design-review，产品Recording仍默认关闭。
