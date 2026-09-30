# Proposal：CU-S4 AX失败后的同窗口视觉降级

关联Story CU-S4与Accepted AD-CU-08。Architecture Impact：architecture-change（CUA Adapter Observe路由与TM-S9运行时Observation传递）；不改SQLite、不替换Driver、不新增执行栈。

## Why

正式 QQ 音乐与企业微信样本中，可信窗口已建立，但自绘/WebView界面返回空或不稳定AX元素；后台动作失败后又没有截图，归属慢脑无法形成坐标候选。TM-S9同时丢弃`UnknownObserved`携带的视觉Observation，顶部步骤还可能把已Observe失败误投影为完成。

## What Changes

- 保持AX优先；空AX、目标多义或动作未确认时只补采同一可信窗口截图。
- 已取得的`UnknownObserved`视觉证据穿透内存运行时返回Gateway。
- 通用`computer.step`窗口坐标文本由Worker强制精确target与foreground。
- 已Observe失败步骤投影为未核实，不显示完成图标。
- 通用桌面任务由慢脑一次提交完整计划片段，顶部浮窗展示全部槽位及执行状态。
- 建立隔离回归、Rust测试与正式QQ音乐macOS验证。

## Non-goals

不截图常开、不采全桌面、不重试动作、不增加视觉模型或第二Driver、不扩大消息发送确认范围；Windows继续暂缓。
