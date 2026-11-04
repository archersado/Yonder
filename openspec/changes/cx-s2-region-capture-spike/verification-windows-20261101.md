# Windows Verification Goal：圈选捕获核心样本

日期：2026-11-01
结论：**PASS（单显示器核心捕获子范围）；完整 Windows 同样本仍未通过。**

## 通过范围

- 在 Windows 11 x64、150% DPI 上使用公开 `Windows.Graphics.Capture`，只捕获探针自己创建的无标题四色窗口。
- 240×160 逻辑像素连续三次均返回 360×240 像素；四色中心采样、逻辑/物理尺寸换算和 `DisplayConfig` 活动路径预检均通过。
- 每次校验后关闭 frame/session/pool 并释放 D3D staging texture；没有保存截图、窗口标题或像素，没有安装鼠标/键盘监听，也未接入产品 Port、Gateway 或任务存储。
- MSVC `/W4` 编译通过；探针为短生命控制台进程，退出码可由 PowerShell 判定。

证据：

- `spikes/region-capture/evidence/windows-20261101/result.json`
- `spikes/region-capture/windows-probe.cpp`
- `spikes/region-capture/build-windows.ps1`

## 未通过范围与原因

- 本机只有一个显示器，`negative_origin_available=false`，不能证明副屏、负坐标、不同 DPI 显示器或物理断开。
- 当前探针直接绑定自身公开检查窗口，不弹系统选择器、不安装输入监听；因此不能替代物理鼠标圈选、物理 Esc、超时清场或切屏过程。
- 本次公开窗口捕获没有触发权限失败，尚未覆盖未授权、运行中撤销和受保护窗口拒绝。

## 解决方案

连接第二显示器并配置负坐标/不同缩放，复用同一四色窗口分别运行；另建不持久化像素的显式选择生命周期探针，由用户物理完成圈选和 Esc，并在系统隐私设置中执行撤权样本。上述证据通过前，AD-CX-01 仍只接受 macOS Preview，不把产品圈选 Port 或默认权限请求扩展到 Windows。
