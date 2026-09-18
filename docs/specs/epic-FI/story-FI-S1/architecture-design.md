# FI-S1 架构设计

## 边界与依赖

可信File Adapter规范身份，处理大小写、软链接和硬链接；同文件写串行，依赖TM-S2。候选身份与提交语义见Proposed AD-FI-01，双平台Spike通过前不得接Gateway。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

macOS候选身份为`st_dev + st_ino`，Windows候选为volume serial + file index。新输出规范化既有父目录；临时文件必须与目标同目录。Yonder写租约与Office/WPS宿主锁均须满足，不能以其中一个替代另一个。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。
