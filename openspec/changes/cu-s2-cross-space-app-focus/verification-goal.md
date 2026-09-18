# 独立 Verification Goal：跨Space应用前置

状态：PASS（macOS，2026-09-17）。

目标：macOS真实应用位于其他桌面工作上下文时，证明`launch_app`保持后台语义；同任务显式`bring_to_front`使用SDK返回的可信身份，后置Observe确认目标可见；失败不得伪报成功。不得使用AppleScript、硬编码Dock坐标或第二执行栈。Windows按用户决定暂缓。

结果：从VS Code前台创建真实Yonder任务`task_b24c51cb03a47731ff0587e813575108`。`launch_app(com.tencent.WeWorkMac)`成功且`target_visible=false`；同一任务显式`bring_to_front`成功，后置SDK窗口Observe返回`target_visible=true`；任务最终`completed@10`。Worker以SDK已校验bundle id解析主进程，兼容启动器PID交接和窗口重建；未使用系统脚本、硬编码坐标或第二CUA执行栈。

证据：`apps/desktop/evidence/cu-s2-cross-space-20260917/result.json`与`wecom-front.png`。Windows继续按用户决定暂缓，因此完整CU-S2仍为verifying，本增量可以关闭。
