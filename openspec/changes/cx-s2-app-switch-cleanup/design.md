# 设计

Desktop组合根只监听既有`region-preview`窗口的原生`Focused(false)`事件，并用Yonda自身的macOS活跃态确认它已不再前台；不安装全局鼠标、键盘或工作区监听。确认卡恢复时重新取得窗口焦点。仅窗口仍可见且并非捕获阶段时调用共享清场逻辑；清场统一取消Region语音、清除Application `PreviewSession`、向确认卡发出`yonda-region-clear`、隐藏窗口并写入有界`app-switch`原因。

区域截图前既有隐藏命令先将会话原子推进到`capturing`，再隐藏窗口；该阶段的失焦事件被忽略，避免原生事件先于可见性更新时误清场。`selecting → capturing → reviewing`保持原有语义。重复失焦与关闭事件幂等，均不发送Agent输入、不写任务事件或日志正文。

验证使用真实macOS窗口：选择中切到另一应用、确认卡中切到另一应用均清场；主动隐藏后截图仍可进入确认卡；验证输出只含布尔结果。
