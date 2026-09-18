# DS-S1 持续展开资源预算独立验证

Story: DS-S1
OpenSpec: e0-validate-desktop-foundation
日期：2026-09-13
依据：ARCHITECTURE-SPINE「非功能与发布」、AD-E0-01。
状态：采样完成，内存预算未通过；返回 Spike 性能修复阶段，不 Archive。

## 方法与环境

沿用 PID 66418 的轻量桌宠构建，没有产品代码改动。复用 measure-app-macos.py 的系统 resource coalition 归属及 libproc 统计，新增 --awake-probe 验证模式。awake-probe-macos.swift 编译至临时目录，只读取指定 Yonda 窗口；开始前以及约120/240秒经既有托盘“显示小龙”保持展开。每5秒核对尺寸和应用未隐藏，约0/5/150/300秒保存仅小龙窗口截图，不额外录制桌面。

包含两次保持展开交互的开销，不能称为完全无交互动画基准；onscreen 标记仅记录。窗口尺寸不等于每一帧都发生动画，因此额外人工核对开始两张及结束截图，开始姿态有变化，末次小龙完整。没有篡改三分钟计时或产品动画频率。

命令：`python3 spikes/desktop-foundation/measure-app-macos.py 66418 spikes/desktop-foundation/evidence/app-animated-budget-20260913.json --awake-probe /private/tmp/yonda-awake-probe`。退出0表示采样成功，不表示性能通过。

## 证据与结果

原始数值：`spikes/desktop-foundation/evidence/app-animated-budget-20260913.json`；四张窗口截图在同名 `.frames/` 目录。Swift探针实际编译成功，采样器自检通过；事后检查61个样本、同一窗口编号、全部尺寸为200至201的CG边界且应用未隐藏、两次中途唤醒、四进程启动身份一致。

实际时长 300.014 秒。

| 进程角色 | 平均 CPU（单核） | 平均 footprint MiB | 峰值 footprint MiB |
|---|---|---|---|
| host | 0.0480% | 25.64 | 26.00 |
| com.apple.WebKit.GPU | 0.0620% | 70.91 | 83.72 |
| com.apple.WebKit.Networking | 0.0005% | 6.51 | 6.58 |
| com.apple.WebKit.WebContent | 0.0642% | 60.94 | 67.41 |

四进程总 CPU **0.1747%**；同一采样点物理内存求和平均 **163.99 MiB**、峰值 **182.72 MiB**。峰值为同时点总和的最大值，并非四个独立峰值相加。

## 结论与后续

CPU 读数低于1%目标。内存求和读数超过150 MB目标，**不能判性能通过**。该值不是系统去重后的coalition内存，不直接断言真实独占内存等于求和，也不能以可能共享为由忽略超预算信号。应先核对渲染层/图像解码与GPU占用，做同口径前后对照；禁止为通过测量而关闭眨眼、呼吸等既定功能。

与上一轮自然闲置相比，主要增量来自WebKit GPU及WebContent，宿主仍约26 MiB。此比较提示优化方向，不是对某个CSS规则或素材的根因证明。下一步先定位后修改，不盲目降低动画帧率。

本轮无Windows、系统WindowServer整体开销或任务负载验证。DS-S1/AD-E0-01继续未完成。
