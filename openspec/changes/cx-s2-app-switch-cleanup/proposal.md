# Proposal：CX-S2 应用切换清场

关联Story：CX-S2；关联决策：Accepted AD-CX-01。

## Why

圈选层与确认卡在应用切换时尚无明确的原生清场路径，临时截图或语音会话可能继续保留，不能满足CX2-02的切屏清理要求。

## What Changes

macOS中，只在圈选会话打开期间于AppKit主线程订阅前台应用切换通知；切换到其他应用时取消本轮语音、清除PreviewSession、清空WebView并隐藏窗口。截图主动隐藏前会先进入捕获态，完成后回到确认卡；不会发送输入、创建任务、轮询状态或安装常驻全局监听。

Architecture Impact：conforming；不新增协议、持久化、权限或依赖。Windows继续按用户决定暂缓。
