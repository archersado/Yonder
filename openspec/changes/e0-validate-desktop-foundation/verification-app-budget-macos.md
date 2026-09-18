# DS-S1 macOS 全应用资源采样独立验证

Story: DS-S1
OpenSpec: e0-validate-desktop-foundation
日期：2026-09-13
依据：ARCHITECTURE-SPINE「非功能与发布」、AD-E0-01。状态：本轮自然闲置采样完成；不 Archive。

## 目标与归属

纠正旧报告仅统计宿主的缺口。macOS 的 WebKit 辅助进程 PPID=1，不能用父子关系归属。launchctl print pid/<PID> 的 resource coalition 区块提供系统归属 ID、名称和 bundle ID。本轮四个进程完全匹配宿主的同一资源分组，bundle ID=com.yonder.e0-spike；不是仅按进程名称或相邻 PID 推测。

使用系统 proc_pid_rusage 的 rusage_info_v0 读取累计用户/内核 CPU 纳秒、resident 和 physical footprint；不采集页面或应用正文。采样脚本 measure-app-macos.py 只用 Python 标准库及系统 libproc，无产品代码改动。每次枚举当前 WebKit，启动时间/成员变化即标记不完整，不把异常降级为仅统计宿主。

## 执行与数据

命令：`python3 spikes/desktop-foundation/measure-app-macos.py 66418 spikes/desktop-foundation/evidence/app-budget-20260913.json`，退出 0。脚本 `--self-test` 的结构尺寸及分组解析检查通过；事后验证 61 个样本、四个固定成员及启动身份全程一致。

实际时长 300.027 秒，每 5 秒采样一次。宿主二进制 SHA-256：`b9feea3ba691dd61f82e0f25853e30845f73e2e6a9d4efc0485907225ecdfe32`。原始数值见 `spikes/desktop-foundation/evidence/app-budget-20260913.json`。

| PID | 角色 | 平均 CPU（单核） | 平均 footprint MiB | 峰值 footprint MiB |
|---|---|---|---|---|
| 66418 | host | 0.0031% | 25.04 | 25.25 |
| 66439 | com.apple.WebKit.GPU | 0.0051% | 28.10 | 58.06 |
| 66440 | com.apple.WebKit.Networking | 0.0002% | 6.47 | 6.48 |
| 66441 | com.apple.WebKit.WebContent | 0.0035% | 48.35 | 48.58 |

合计平均 CPU **0.0120%**；同时点的 footprint 总和平均 **107.96 MiB**，峰值 **138.07 MiB**（约 144.77 MB）。总峰值取每个样本的求和最大值，不是各进程各自峰值相加。

## 结论与限制

本轮没有由测试工具点击/唤醒小龙，保留正常三分钟闲置行为；没有持续展开或同步记录每个样本的可见/休眠状态，不能据此宣称持续动画达标。此次自然闲置 CPU 与分进程物理内存求和读数低于既定 1%/150 MB 预算，但求和不是系统去重后的 coalition 内存，也不能代替所有目标场景的性能结论。

测量仅覆盖宿主及同组 WebKit 进程，不含系统共享 WindowServer 开销；无录制、外部任务负载或 Windows 复验。完整持续动画预算、Windows、宿主 IPC/权限拒绝等门禁保留。DS-S1 与 AD-E0-01 未完成。
