# AD-CM-01 结构化命令执行与进程树停止

状态：Proposed  
日期：2026-09-17  
关联：CM-S1

## 决策问题

Yonder需要执行`program + args + cwd + env`，限制输出并在超时或取消时停止完整进程树。Shell只能由独立、显式且经用户确认的入口请求，不能由结构化命令参数隐式拼接。

## 候选决策

产品优先采用Rust标准库`std::process::Command`。macOS/Linux子进程建立独立进程组并向进程组发送停止信号；Windows使用Job Object并在关闭时终止其全部进程。stdout/stderr边读边计数，超过预算立即停止进程树，只返回有界前缀和截断事实。程序和cwd须为规范化绝对路径；环境采用最小继承加显式覆盖。

结构化入口不得接受命令字符串、重定向、管道或命令替换。Shell是后续独立能力，需要可信本地确认事实，Agent不能自报确认。提权、安装、删除、支付和发送同样必须先取得用户确认。

## Spike门禁

先以统一样本验证：参数不经Shell解释、超时停止父子进程、stdout/stderr预算、退出码与启动失败分类。macOS与Windows证据齐备前保持Proposed；当前Windows按用户决定暂缓，因此不得开放产品Gateway或宣称CM-S1完成。
