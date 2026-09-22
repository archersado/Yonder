# 设计

会话成功显示后，macOS Adapter在AppKit主线程订阅`NSWorkspaceDidActivateApplicationNotification`；通知目标不是本应用时回到既有共享清场函数。清场、发送和打开失败都会注销观察者。Application仍是会话、语音和附件清场的唯一所有者。该观察者只在会话可见期间存在，不轮询数据库。重复切换、关闭事件幂等，均不发送Agent输入、不写任务事件或日志正文。

区域截图前既有隐藏命令先将会话原子推进到`capturing`，再隐藏窗口；应用前台身份没有变化，因此不会触发工作区切换通知。`selecting → capturing → reviewing`保持原有语义。

验证使用真实macOS窗口：选择中切到另一应用、确认卡中切到另一应用均清场；主动隐藏后截图仍可进入确认卡；验证输出只含布尔结果。
