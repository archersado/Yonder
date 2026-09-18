# 正式桌面私有stdio研发接入

## ADDED Requirements

### Requirement: 显式研发连接共享宿主
普通启动不得读取stdin；--local-agent-stdio必须绑定固定研发Agent，使用正式系统目录唯一TaskHost，不得从请求选择身份。Agent先hello，再创建和查询真实任务。

### Requirement: 有界连接
完整帧最多64KiB，坏帧或EOF结束连接；完整协议响应走Rust编码，GUI不等待阻塞读线程退出，正文不进诊断。父进程私有管道只构成研发边界，不宣称生产认证。

### Requirement: 原生真实菜单
Agent登记任务为created，悬停小龙显示真实任务，移入面板保留，移出消失；不伪造执行态。Windows与生产Socket门禁保留。
