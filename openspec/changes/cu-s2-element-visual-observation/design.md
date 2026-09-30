# Design：元素解析与临时窗口Observation

Node Worker在同一受监管Driver会话中保存`launch_app`已验证的bundle id，并在每步前刷新PID与普通窗口。窗口级动作调用`get_window_state(include_screenshot=false)`取得新鲜元素；需要语义文本目标且只有一个可用编辑元素时，注入SDK元素引用。零个或多个匹配时不调用副作用工具，改为对同一PID/window执行一次截图Observe。

Worker以`action_known=true`、`action_effect=refused`和`observe_valid`区分“动作未执行但观察有效”与Worker崩溃。Application新增只读的`UnknownObserved`分支：attempt仍按原规则记为unknown，视觉事实只在当前调用栈返回。计划执行保留最后一个临时Observation并通过协议1.34响应给归属慢脑；1.31～1.33调用方不接收新增字段。

截图由既有受控证据目录持有，限制4MiB和PNG/JPEG/WebP。它不进入任务库、事件、Outbox、日志或Jev输入，任务终结/会话回收继续使用既有清理。系统安全界面、隐私窗口与用户排除应用沿用CU-S2采集门禁。
