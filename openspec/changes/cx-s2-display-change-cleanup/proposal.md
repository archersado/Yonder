# Proposal：CX-S2 显示器参数变化清场

关联Story：CX-S2；关联决策：Accepted AD-CX-01。

## Why

主显示器分辨率变化后，圈选层或确认卡可能残留。需要以确定性的原生屏幕参数通知触发共享清场，不依赖其他应用偶然激活或窗口关闭事件。

## What Changes

macOS中，只在圈选会话打开期间订阅`NSApplicationDidChangeScreenParametersNotification`。Windows在同一生命周期内创建消息窗口，只接收`WM_DISPLAYCHANGE`；显示拓扑、分辨率或显示器接拔导致该消息时执行清场，不订阅范围过宽的通用设备变化消息。收到通知后都取消本轮语音、清除PreviewSession、清空WebView并隐藏窗口；所有既有清场、发送和打开失败路径都会注销观察者并销毁平台资源。Windows观察器无法启动时失败关闭，不保持无保护的圈选会话。

Architecture Impact：conforming；不新增协议、持久化、权限或产品技术栈。Windows仅显式使用锁文件中已有的`windows-sys`系统绑定。
