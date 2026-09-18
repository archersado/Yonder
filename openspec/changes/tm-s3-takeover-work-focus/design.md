# 设计

CUA动作后，宿主使用同一可信WorkTarget捕获并内存保留WorkRef。安全步骤边界上的takeover控制原子转paused/stopped；宿主释放任务占用并设置桌面接管闸，再依次提交locating、调用WorkFocusPort、提交focused或failed。每次阶段写入与任务sequence、事件、Outbox同事务。

协议1.13只增加可选定位结果；旧版本投影移除。PID、窗口ID、AX对象不落库、不接受UI或Agent输入。目标失效、不唯一、权限、激活或核验失败保持paused，不重试、不改选同名窗口。Recording、交回与Windows不在本增量。
