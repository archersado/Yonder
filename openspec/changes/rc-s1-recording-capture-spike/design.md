# 设计

独立探针复用系统原生 CGEventTap，不复用或扩展产品 CUA Worker。回调只把事件种类、来源 PID/可信标记和单调序号放入固定容量队列；隐私检查与分类在回调外完成。证据只保存 accepted/excluded/gap/stopped 计数和稳定原因，不保存输入正文、AX 文本、截图或坐标。

先验证来源分类与停止边界，再验证 Secure Text Field、系统安全界面和排除应用。任何样本不能稳定拒绝即淘汰该路线，不用启发式补齐。Windows 使用相同样本但按用户决定暂缓，因此本 Change 只能形成 macOS 子结论。

