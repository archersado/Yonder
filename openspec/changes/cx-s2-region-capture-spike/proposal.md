# Proposal：CX-S2 圈选提问原生捕获 Spike

关联Story：CX-S2；关联AD：Proposed AD-CX-01。

使用统一无敏感检查图形比较Windows/macOS原生显示器坐标和区域截图路线，验证高DPI、多显示器、负坐标、权限及异常清场。Spike不接产品UI、Application Port、Gateway、语音或任务存储，不保存屏幕内容。

Architecture Impact：architecture-change（新增显式屏幕区域上下文候选边界）。依AD-CX-01限于隔离Spike，双平台证据与ADR接受前不授权产品Apply。
