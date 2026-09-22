# Proposal：CX-S2 显示器参数变化清场

关联Story：CX-S2；关联决策：Accepted AD-CX-01。

## Why

主显示器分辨率变化后，圈选层或确认卡可能残留。需要以确定性的原生屏幕参数通知触发共享清场，不依赖其他应用偶然激活或窗口关闭事件。

## What Changes

macOS中，只在圈选会话打开期间订阅`NSApplicationDidChangeScreenParametersNotification`；收到通知后取消本轮语音、清除PreviewSession、清空WebView并隐藏窗口。所有既有清场、发送和打开失败路径都会注销该观察者。Windows继续按用户决定暂缓。

Architecture Impact：conforming；不新增协议、持久化、权限或依赖。
