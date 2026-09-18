# macOS 来源分类 Verification Goal

日期：2026-09-17  
结论：FAIL（CGEvent来源字段路线淘汰）

## 目标

验证 listen-only CGEventTap 能否只凭事件来源字段区分真实用户候选与程序注入，且证据不保存输入正文、坐标、截图或AX文本。

## 结果

探针自检通过有界容量、停止边界和静态来源分类。真实桌面对照中使用 CUA REPL 点击 Yonda，一次程序注入产生51个已监听事件；其`source_pid/source_tag`全部落入用户候选，`injected_excluded=0`。结构化证据见[结果](../../../spikes/recording-capture/evidence/source-macos-20260917/result.json)。

因此CGEvent来源字段不能满足REC-03，按AD-RC-01淘汰门槛拒绝进入产品。后续只验证原始IOHID事件与CGEvent的有界时间关联；若仍不能稳定证明硬件来源，则RC-S1不得采集输入轨迹。
