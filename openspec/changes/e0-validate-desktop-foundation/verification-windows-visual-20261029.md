# 独立 Verification Goal：DS-S1 Windows 原生视觉与透明边缘

日期：2026-10-29
Story：DS-S1
Goal：`01a15eda-f12a-7e32-bd6c-2d3336e17351`
结论：PASS（Windows 当前素材原生视觉、尺寸与透明边缘子范围）

## 隐私受控方法

验证器先创建600×600纯色背景窗口，颜色固定为RGB `(220,230,242)`，再只启动并移动自己的Yonda测试进程到该背景内部。截图仅覆盖Yonda窗口，不包含用户桌面其他区域。发现同路径已有Yonda进程时脚本拒绝运行。

本机DPI为144（150%）：Win32窗口几何是200×200逻辑像素，对应截图区域300×300物理像素。验证器按窗口DPI换算后使用 `CopyFromScreen` 截取物理区域，并分别检查四角背景色、背景像素占比和非背景角色像素占比。

## 结果

- 原生窗口：`(640,400)`、200×200逻辑像素；物理截图：`(960,600)`、300×300像素。
- 四个角像素全部精确为受控背景色，透明区域实际透出背景。
- 90,000个物理像素中，61,492个匹配受控背景，28,508个属于角色或状态UI；透明边缘和可见内容门槛均通过。
- 人工查看原生截图：当前小龙完整可见，角色四肢、翅膀、尾巴和面部未被裁切；周围没有白色或黑色矩形底；右下角连接状态圆点正常可见。
- 截图：`apps/desktop/evidence/ds-s1-windows-20261029/native-visual.png`。
- 结构化结果：`apps/desktop/evidence/ds-s1-windows-20261029/native-visual-result.json`。
- 可重复脚本：`apps/desktop/evidence/ds-s1-windows-20261029/capture-native-visual.ps1`。

## 验证工具失败与修正

第一次截图错误地把PowerShell取得的DPI虚拟坐标直接传给消费物理像素的 `CopyFromScreen`，因此抓到窗口外的200×200白色区域。空白图和结果分别保留为 `native-visual-dpi-failure.png`、`native-visual-dpi-failure-result.json`。最终脚本读取窗口DPI并截取300×300物理区域；首次结果属于验证方法失败，不是产品视觉失败。

## 判定边界

结合 `verification-windows-drag-20261029.md`，Windows当前素材的原生视觉、200×200尺寸、透明边缘与原生合成输入拖动技术链路已有证据。用户亲手拖动、九态全部状态的原生事件驱动与视觉、性能及完整生命周期仍单列；macOS剩余门禁不变，DS-S1总Goal未完成，不Archive。本轮未执行或修改CUA。
