# DS-S1 同进程IPC版本资源预算独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

目标：新增IPC验证线程后重新测量整个应用，不能沿用旧二进制性能结论。状态：本机单次持续展开几何场景的资源读数达标；同期视觉证据缺失，整体Goal未通过。

## 方法与证据

当前.app PID92356，二进制SHA-256为`1cdf0a38eebbc3f9c3c47fa1c0a0441b337d4597af327d6421e2f1a50f4dcb2a`，与已验证宿主IPC的产物相同。沿用measure-app-macos.py的resource coalition归属、libproc计数、每5秒一次采样、约120/240秒托盘找回保持展开。采样器自检通过。IPC线程空闲，不发送负载，不把本轮作为IPC压力测试。

命令：`python3 spikes/desktop-foundation/measure-app-macos.py 92356 /Users/archersado/workspace/Yonder/spikes/desktop-foundation/evidence/host-ipc-budget-20260913.json --awake-probe /private/tmp/yonda-awake-probe --external-frames`。

脚本退出0。61个样本、300.014778秒；同一窗口26360，全部尺寸200×201且应用未隐藏；四个进程启动身份一致，两次中途托盘找回。使用断言复核这些条件，并写入host-ipc-budget-audit-20260913.json。

## 读数

| 指标 | 加入IPC前 | 当前同进程IPC版本 |
|---|---|---|
| CPU（单核） | 0.1550% | 0.1695% |
| 同时点四进程footprint求和均值 | 117.22MiB | 122.24MiB |
| 同时点求和峰值 | 122.85MiB | 135.55MiB |

当前CPU低于1%，峰值约142.13MB，低于150MB。分进程footprint求和不是系统去重内存，也不含WindowServer整体开销。不同启动缓存/桌面环境意味着表格只能比较两轮读数，不能将全部差值因果归于IPC线程。

## 截图时间核对

独立窗口截图探针退出6，图像实际为横向双眼探头。不能依文件名host-ipc-budget-start-20260913.png声称采样起点画面。

本机文件时间证明采样报告创建于19:58:02、最终写入于20:03:02；截图创建于20:16:32，比采样完成晚约809.95秒。该图不属于五分钟采样时段，既不能证明同期正在休眠，也不能补充同期展开视觉验收。文件时间与原始资源记录一并保留，未修改截图或失败返回码。

因此本轮通过范围为原生几何保持展开情况下的全应用资源读数，未把它扩大为五分钟逐帧动画验证。现有视觉/动画证据仍按各自版本和场景追溯。Windows、任务负载、跨用户权限与完整状态事件链路门禁仍保留。
