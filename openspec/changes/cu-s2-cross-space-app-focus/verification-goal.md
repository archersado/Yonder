# 独立 Verification Goal：跨Space应用前置

状态：PASS（macOS，2026-09-17）。

目标：macOS真实应用位于其他桌面工作上下文时，证明`launch_app`保持后台语义；同任务显式`bring_to_front`使用SDK返回的可信身份，后置Observe确认目标可见；失败不得伪报成功。不得使用AppleScript、硬编码Dock坐标或第二执行栈。Windows按用户决定暂缓。

结果：从VS Code前台创建真实Yonder任务`task_b24c51cb03a47731ff0587e813575108`。`launch_app(com.tencent.WeWorkMac)`成功且`target_visible=false`；同一任务显式`bring_to_front`成功，后置SDK窗口Observe返回`target_visible=true`；任务最终`completed@10`。Worker以SDK已校验bundle id解析主进程，兼容启动器PID交接和窗口重建；未使用系统脚本、硬编码坐标或第二CUA执行栈。

证据：`apps/desktop/evidence/cu-s2-cross-space-20260917/result.json`与`wecom-front.png`。Windows继续按用户决定暂缓，因此完整CU-S2仍为verifying，本增量可以关闭。

## 2026-09-28 回归复测

真实企业微信计划复现了启动后窗口重建：原 Worker 在显式 `bring_to_front` 的前置 Observe 仍使用宿主启动前目标，只在动作完成后刷新已启动应用。实现已改为动作前按已验证 bundle id 刷新 PID 与最大普通窗口，前置 Observe、SDK 参数和后置 Observe 因而使用同一新鲜目标。产品 Worker 独立样本结果为：`launch_app action_succeeded=true`、`bring_to_front action_succeeded=true`、两步 `observe_valid=true`、前置后 `target_visible=true`。

正式 Gateway 四槽位样本没有伪报通过：跨 Space `bring_to_front` 期间 trycua 产生了 5 个 `left-mouse-down`，CoreGraphics 报告来源 PID=0、user-data=0；现有 HID 监控无法把它与真实物理点击可靠区分，因此任务按 `unknown/user-input` 转为 `interrupted`，未继续执行按键或发送。历史 PASS 证据保持有效，但当前系统版本上的完整跨 Space Gateway 复测属于安全阻塞；不得用时间窗口或忽略 PID=0 点击规避。Windows继续暂缓。
