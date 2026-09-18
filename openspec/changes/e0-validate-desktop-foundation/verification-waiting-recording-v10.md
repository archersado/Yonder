# 等待用户与手动录制动画 v10 验证

日期：2026-09-15  
关联 Story：DS-S1  
结论：PASS（独立动画预览与 macOS 构建；录制真实事件仍未接线）

## 结果

- 等待用户按 `0→1→2→3→3→2→1→0` 播放四张完整透明姿态，ego-browser 长周期采样捕获全部四帧；抬手前后截图无局部手臂拼接和圆形提示。
- v10 运行时副本保留原 alpha，只去除三个抬爪帧中的粉色圆形掌垫；v4 原图未覆盖。
- 手动录制持续显示红点 `REC`；完整身体承载双爪，相机仅做约 1px 快门反馈，局部闪光峰值超过 0.7。
- 九态呼吸、眨眼和各自动作循环通过；录制切换离开后相机与闪光清理通过；减少动态效果下相机和 `REC` 保留静态、闪光为 0。
- `cargo build --offline --locked -p yonder-desktop` 通过；预览应用重新打包并启动，重启前任务库仅有 4 个 `cancelled` 任务。

## 证据

- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/result.json`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/waiting-down.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/waiting-up.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/recording-flash.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/recording-reduced.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/top.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/bottom.png`
- `apps/desktop/evidence/state-assets-v4-20260915/waiting-recording-v10/light.png`

Windows 证据依用户决定继续暂缓。本验证不创建任务、不启动系统录制、不宣称五个预览态已连接真实事件。
