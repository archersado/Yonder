# Tasks

- [x] 接受AD-CU-09并同步CU-S4三份设计
- [x] 将组合根、Adapter与Worker切为Sky单栈
- [x] 移除trycua生产依赖、打包资源与环境回退
- [x] 覆盖应用绑定、transcript解析、语义动作与包边界测试
- [ ] 构建正式macOS Yonder并复验QQ音乐多步骤链路
- [ ] 建立并通过独立Verification Goal
- [ ] Windows对等验证（依用户决定暂缓）

2026-10-01正式验证返回Apply：包边界与隔离Worker通过，但正式Yonder在`list_apps`前的Sky native-pipe握手被服务关闭，阶段为`transport-closed`。2026-10-04进一步确认固定Sky发行物提供官方签名MCP Client，且Client要求OpenAI Team ID父进程；同发行物签名Node派生Client的`list_apps/get_app_state`已通过隔离探针。产品实现改走该签名MCP桥，仍须完成正式Gateway与独立Goal后才能合入dev。

2026-10-04复核：签名MCP桥、身份审计和隔离Worker已通过；正式Gateway进入Sky后因macOS当前锁屏返回`target-window-unavailable`并安全交回，任务已取消。解锁后仍须完成QQ音乐正向与视觉交回样本；此前本Change继续Apply。
