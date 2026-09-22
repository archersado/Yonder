# macOS 排除应用 Verification Goal

日期：2026-09-22  
结论：子范围 PASS。

探针读取当前前台应用PID并作为排除目标，随后注入一条带Yonder标记的零位移滚动事件。结果为`controlled_session_inputs=0`、`excluded_app=1`、`known_injected_rejected=0`、`gaps=0`；没有采集事件、按键、坐标、截图、AX文本或完整事件Payload。样本未改变前台焦点，也未保存应用名称或PID。

该结果只证明Spike的PID排除路径可用，不授权产品Recording、持久化或Replay；Windows同样本继续按用户决定暂缓。

结构化证据：[result.json](../../../spikes/recording-capture/evidence/excluded-app-macos-20260922/result.json)。
