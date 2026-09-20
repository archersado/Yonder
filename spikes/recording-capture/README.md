# Recording Capture Spike

关联 RC-S1、Proposed AD-RC-01 与 `rc-s1-recording-capture-spike`。探针只输出来源分类计数，不保存按键、坐标、AX正文或截图。

```bash
clang -x objective-c -std=c11 -Wall -Wextra -Werror -framework AppKit -framework ApplicationServices -framework Carbon -framework CoreFoundation -framework IOKit spikes/recording-capture/macos-probe.c -o /private/tmp/yonda-recording-macos-probe
/private/tmp/yonda-recording-macos-probe --self-check
/private/tmp/yonda-recording-macos-probe 5
/private/tmp/yonda-recording-macos-probe --controlled 5
/private/tmp/yonda-recording-macos-probe --outside 5
/private/tmp/yonda-recording-macos-probe --hid-listen 10 20
swift spikes/recording-capture/secure-input-fixture.swift
```

2026-09-17真实CUA注入对照表明`source_pid/source_tag`会落入用户候选，纯CGEvent来源字段路线已淘汰。IOHID关联可拒绝程序注入，但三轮真实物理正样本在20/100ms窗口内均无法稳定证明用户来源，也已淘汰。

2026-09-18 探针改按用户控制租约计数：`--controlled`只验证租约内的无正文受控会话输入，`--outside`必须保持零输入；它们不证明物理用户来源。隐私排除、已知注入、停止与队列样本仍是产品实施前门禁；不得接产品任务库。
