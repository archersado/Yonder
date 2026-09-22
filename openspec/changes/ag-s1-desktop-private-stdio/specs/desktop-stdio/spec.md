# 正式桌面私有stdio研发接入

## ADDED Requirements

### Requirement: 显式研发连接共享宿主

桌面进程 MUST 只在显式传入`--local-agent-stdio`时启用研发连接，并绑定固定研发Agent与唯一TaskHost。

#### Scenario: 显式启用研发连接

普通启动不得读取stdin；--local-agent-stdio必须绑定固定研发Agent，使用正式系统目录唯一TaskHost，不得从请求选择身份。Agent先hello，再创建和查询真实任务。

### Requirement: 有界连接

研发连接 MUST 以64KiB完整帧为边界，坏帧或EOF结束当前连接，且不泄漏请求正文。

#### Scenario: 坏帧关闭连接

完整帧最多64KiB，坏帧或EOF结束连接；完整协议响应走Rust编码，GUI不等待阻塞读线程退出，正文不进诊断。父进程私有管道只构成研发边界，不宣称生产认证。

### Requirement: 原生真实菜单

研发Agent登记任务后，桌宠 MUST 只显示真实任务状态，不伪造执行态。

#### Scenario: 显示created任务

Agent登记任务为created，悬停小龙显示真实任务，移入面板保留，移出消失；不伪造执行态。Windows与生产Socket门禁保留。
