# Design

Spike固定安装0.25.0和0.30.4到相互隔离的目录，先静态提取工具Schema，再对同一macOS fixture运行候选样本。fixture公开焦点和值的独立读取面；探针从新鲜窗口截图取得窗口局部坐标，优先以0.30.4的`type_text(x,y,text,foreground)`原子聚焦并输入固定标记，再重新Observe。

探针在动作前保存无标题的前台应用身份摘要，在成功、失败和关闭路径分别确认恢复。错误窗口、目标关闭或不可核实结果立即停止；不得自动重放。输出为有界JSON布尔和稳定枚举，不保存截图、标题、正文或完整Payload。

通过仅说明候选SDK在隔离macOS样本满足升级前置；正式包替换、Yonder Gateway、企业微信样本及Windows分别属于后续Apply/验证门禁。
