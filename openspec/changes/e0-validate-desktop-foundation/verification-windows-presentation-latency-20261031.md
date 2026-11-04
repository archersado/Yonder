# 独立 Verification Goal：DS-S1 Windows 核心展示状态延迟（listening 子范围）

日期：2026-10-31；平台：Windows。结论：**PASS（仅 `task.create → listening` 子范围）**。

## 验证方法

- 使用当前 Windows 调试构建启动真实 `yonder-desktop.exe`，以临时登记的本地 Agent 连接正式 Named Pipe Gateway，协商协议 `1.19`。
- 计时起点是验证器向 Gateway 写入真实 `task.create` 请求之前；终点是 pet WebView 在真实 `applyPresentation` 改变状态并同步展示配置后，通过有界白名单诊断首次报告 `phase=presentation-change, task_state=listening`。
- 诊断只包含阶段、窗口模式、白名单状态和毫秒值，单页面最多10条；没有任务正文、截图、输入、命令输出或完整 Agent Payload。
- 任务创建成功后通过正式 `task.cancel` 收口，响应终态必须为 `cancelled`。本轮 `cua_executed=false`、`ego_lite_executed=false`。

## 结果

- 任务：`task_55a96dd68fd3c32401a408be34864763`；临时 Agent：`ds-s1-latency-e2fed45ed0b5`。
- 从请求写入至首次 `listening` 展示诊断的观测上界为 **15.829ms**，低于 Accepted AD-E0-01 的 300ms 当前优化目标。
- `task.create` 完整请求耗时为 29.642ms，诊断出现时响应已返回，因此 15.829ms 是保守的异步日志读取上界，不声称是浏览器逐帧绘制或光子到屏幕时延。
- 正式取消请求耗时 7.181ms，返回任务状态 `cancelled`；验证器随后关闭其拥有的桌面进程并禁用临时 Agent。
- 机器可读结果与复现脚本：`apps/desktop/evidence/ds-s1-windows-20261031/native-presentation-latency-result.json`、`verify-native-presentation-latency.ps1`。

## 既有状态与边界

启动时页面先真实恢复为 `paused`，因为本机库中存在一项此前外部 BUA 探测留下的 `interrupted` 任务；该状态没有被测试脚本伪造或直接修改数据库。新任务取消后，展示还受既有 1.6 秒 `listening` 脉冲以及代表任务选择影响，因此本轮不把“取消后恢复状态”作为 300ms 样本，也不要求错误的固定 `idle`。

本结果只关闭 Windows `task.create → listening` 核心状态应用延迟子范围。`executing`、等待、暂停、终态及其恢复的同口径延迟、完整可用状态、macOS 对应数据和九态完整原生链路仍未完成；不能据此把“完整核心窗口状态更新延迟”、DS-S1 总 Goal 或 Change 标为完成。正式 ego-lite 检查受当前 Agent 环境的 `ego_cli bootstrap` 连接限制未纳入本次通过结论，未继续创建 Browser Task Space，也未执行 CUA。
