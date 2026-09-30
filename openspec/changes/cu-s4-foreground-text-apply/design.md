# Design

Rust协议仍是参数模型唯一来源。`valid_semantic_arguments`和Application副本只接受：受支持click语义的`{x,y}`，或`enter-target-query|draft-message-ref + type_text`的`{x,y}`；所有值必须有限，其他字段拒绝。

Worker识别坐标文本后跳过元素和一次性焦点解析，从Application提供的意图引用展开文本。若SDK工具Schema含`delivery_mode`，Worker强制写入`foreground`；若缺失则拒绝，不降级后台。窗口动作继续注入已启动目标的精确target。动作后观察同一窗口并为视觉动作取得临时截图；只有confirmed且Observe有效才推进。

坐标click同样由Worker注入foreground，解决自绘目标激活和消息框聚焦。`unverifiable`不建立凭据；旧元素路径与confirmed视觉焦点一次消费路径保持兼容。
