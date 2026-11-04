# Windows 原生复测：桌宠自动停靠

日期：2026-10-24
结论：FAIL（自动空闲触发仍未通过）

- 干净 Windows 11 实例等待 190 秒后，桌宠仍保持 200×200；没有自动进入 112×56 或 56×112 停靠尺寸。
- 通过同一 Tauri 命令直接调用 `pet_dock` 时，原生窗口能够缩为 112×56，说明 Windows 可变尺寸修复和原生定位路径有效。
- `pet_task_state` 返回 `[false, "idle"]`，不是任务状态 unknown 门禁导致。
- 已将空闲计时与装饰素材加载、任务展示加载解耦，并增加基于最后交互时间的看门狗；真实 190 秒复测仍未触发，不能宣告通过。
- 进程始终无 TCP 监听。
- 结构化证据：[`idle-dock-result.json`](../../../apps/desktop/evidence/ds-s1-windows-20261024/idle-dock-result.json)。

下一步需为前端空闲调度增加可观察的计时状态/调用日志，再定位 Windows WebView 运行时为何未进入 `hide()`；在此之前保持 Story 未通过。
