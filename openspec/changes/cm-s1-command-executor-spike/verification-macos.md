# 独立 Verification Goal：macOS结构化命令路线

日期：2026-09-17  
结论：PASS（macOS Spike子范围；完整Change未通过）

- `/usr/bin/printf`接收含命令替换语法的字面参数，未创建目标文件，证明结构化参数不经Shell解释。
- 测试夹具创建父子进程组；发送组停止信号后父进程与后代均不可存活。
- 无限输出样本只保存65536字节并标记`truncated=true`。
- 证据：[`result.json`](../../../spikes/command-executor/evidence/macos-20260917/result.json)。

Windows Job Object证据按用户决定暂缓，AD-CM-01保持Proposed；本结果不授权产品Gateway、Shell或CM-S1 Archive。
