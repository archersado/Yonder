# Proposal：CX-S2 应用切换清场

关联Story：CX-S2；关联决策：Accepted AD-CX-01。

## Why

圈选层与确认卡在应用切换时尚无明确的原生清场路径，临时截图或语音会话可能继续保留，不能满足CX2-02的切屏清理要求。

## What Changes

macOS中，圈选窗口可见且失去焦点时视为用户切换应用：取消本轮语音、清除PreviewSession、清空WebView并隐藏窗口。捕获过程中窗口主动隐藏不触发清场；无可见窗口时的失焦也不改变会话。不会发送输入、创建任务或新增全局监听。

Architecture Impact：conforming；不新增协议、持久化、权限或依赖。Windows继续按用户决定暂缓。
