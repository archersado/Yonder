# 设计

Task Space调用`user_takeover(task_id, expected_sequence)`。Rust宿主验证窗口label、任务标识和序号，在Application协议层生成固定kind=takeover的本地请求，再复用现有TaskHost停止/定位编排。JS不能传kind、agent_id、capability、deadline或user_intent_id。

通用`task_query`拒绝takeover请求，取消和只读查询保持原路径。Agent Gateway仍可表达控制意图，但不会获得本地用户意图身份，也不能据此启动Recording。本增量不建立租约，只形成可验证的可信入口。
