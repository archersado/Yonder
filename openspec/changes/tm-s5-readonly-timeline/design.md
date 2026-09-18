# 设计

选择任务时并行读取 `task.step.get` 与 `task.events(after_sequence=0, limit=100)`，共用现有选择轮次，迟到响应不得覆盖新选择。详情读取成功后先显示任务字段；事件读取独立成功或局部失败，不清空详情。

事件按服务端顺序渲染为原生有序列表。`step_declaration` 标为 Agent 声明；`attempt_result.phase=observed` 区分动作成功和未达成，`unknown` 显示协议原因；普通事件显示状态变化。页面不显示 Worker/宿主标识，不解释语义或合并相同文字。

最后事件序号小于任务快照序号时显示未完整加载；首批最多 100 条，不增加分页状态或新窗口。浏览器夹具验证声明、observed、unknown、局部失败和迟到选择保护；原生 macOS 再验证真实任务。Windows按用户决定暂缓。
