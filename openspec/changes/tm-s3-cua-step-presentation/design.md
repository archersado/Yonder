# Design：TM-S3 CUA 规划与执行步骤展示

`TaskHost::cua_control_presentation` 仅在已验证的执行提示、归属 Agent 与任务状态均有效时读取计划片段。对 `PlanSlot`，读取其不可变槽位标签并限制至四条；对单步请求，仅传递已校验的步骤标签。`CuaControlHub` 保存该短生命周期投影，`CuaControlPort` 在每次真实派发前标记当前 step_id。

受限 `cua-control` Webview 通过 Tauri command 拉取 Hub 快照。该命令无写能力、不能接受任务身份或步骤内容；UI 不可提交 sequence、计划、动作或执行状态。接管/结束清空 Hub，连接结束不得保留旧任务投影。
