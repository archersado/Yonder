# Design：CX-S2 截图隐藏激活竞态修复

应用切换通知仍由既有 AppKit Adapter 产生，Desktop 宿主继续拥有 `PreviewSession` 生命周期。回调在派发清场前只读取一次 Application 层阶段：若为 `Capturing`，说明覆盖窗口已经由 `region_preview_hide_for_capture` 主动隐藏，本次前台应用变化属于截图事务内部步骤，直接忽略；其他阶段沿用共享清场路径。

判断不读取图片字节、来源应用正文或 Agent 输入，不建立新状态所有者。捕获完成后会进入 `Reviewing`，此后真实应用切换仍清除临时截图。锁失败按既有安全方向继续清场，不把异常当成捕获豁免。
