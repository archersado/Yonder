当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：E0-S1 macOS 基础能力续验

日期：2026-09-11。关联 E0-S1 AC 1–5、同 Change 的托盘找回/退出、本地 IPC 和性能场景、AD-E0-01。Architecture Impact：conforming。状态：部分通过，不能 Archive，ADR 不转 Accepted。

## 本轮变更

原先托盘只有对象初始化，无图标和菜单。补充原生 Y 模板图标及“显示小龙 / 退出 Yonda”；找回事件沿用现有唤醒流程，退出调用 Tauri app.exit。无新依赖、协议、持久化或任务状态所有者。

## 已验证

- macOS 26.5.1、Apple Silicon、Rust 1.98.1。锁定依赖的 IPC Release 构建通过（9.53 秒）；桌面 Release 构建通过（10.68 秒）。两个 Cargo.lock 均未改变。
- `cargo test --release --offline --locked --manifest-path spikes/desktop-foundation/Cargo.toml`：协议往返 1 项通过。
- 同命令指定 `src-tauri/Cargo.toml`：任务门禁与屏幕边界 1 项通过。
- `python3 spikes/desktop-foundation/check-ipc.py`：真实 Rust 服务端/客户端 UDS 往返，版本 0.1、合成 payload macos-e0；服务端退出码 0，端点自动清理。lsof 在服务端等待连接时确认该进程没有 TCP 套接字，临时父目录权限 0700。这是独立 IPC Spike，不是桌面宿主内的 Gateway 接线，也不是跨用户攻击验证。
- ego-browser TaskSpace 10 调用保存的 check-motion.mjs：moving/wagging/stopped 均为 true；派发托盘找回事件后 active 恢复。已关闭 TaskSpace。此测试只覆盖前端接收事件，不代替真实托盘点击。
- 正常终止旧实例后，重新封装并启动本机 .app，PID 93509。原生页面 decode 和同源样式成功；14 秒动画采样 breathing=true、blinks=2、reduced=false。3–5 秒随机间隔下 14 秒出现 2 次闭眼符合范围。

## 五分钟采样

运行 `measure-macos.py 93509 spikes/desktop-foundation/evidence/animations-macos/host-resources-20260911.json`，每 5 秒记录累计 CPU 时间和 RSS，共 61 个点。CPU 使用首尾累计时间差除以实际经过时间计算，不使用瞬时 %CPU 的平均值。结果写入同一 JSON 的 summary。最终实际时长 300.011 秒、61 个样本：宿主 CPU 0.670%，平均 RSS 88.26 MiB、峰值 104.61 MiB。进程在整个采样期间保持运行。

采样只归属桌宠宿主，不包含由系统管理的 WebKit 辅助进程；保留自动 3 分钟休眠，因此是自然闲置生命周期混合采样，不是五分钟持续展开动画预算。不得据此宣称整个应用达到 CPU <1%、内存 <150 MB。

## 尚未通过

- 本轮 CGWindow 查询返回空列表，菜单栏截图无有效内容。不能证明托盘视觉、点击找回、点击退出或真实 Space 切换已通过；不将该截图作为通过证据。
- 尚缺 WebKit 全进程资源归属、持续动画预算、准确冷启动和窗口事件延迟。
- 尚缺桌面宿主与 IPC 合并运行、真实跨用户权限拒绝，以及退出后的完整受管进程清理。
- Windows 没有复测本轮新 UI。历史 Windows IPC/资源数据保留，但不替代当前版本证据。
- 真实后台任务事件源尚未接线。E0-S1/AD-E0-01 未通过前，不开展依赖该桌面技术路线的 OCT-S1 宿主实施。

下一步：在可操作的 macOS 桌面补原生托盘/退出与资源证据，并在 Windows 复验当前增量；满足既定门槛后评审 ADR，再接 OCT-S1 宿主、Credential Store、Gateway/CLI 和同源任务展示。
