# Tasks

- [x] 接受AD-CU-09并同步CU-S4三份设计
- [x] 将组合根、Adapter与Worker切为Sky单栈
- [x] 移除trycua生产依赖、打包资源与环境回退
- [x] 覆盖应用绑定、transcript解析、语义动作与包边界测试
- [ ] 构建正式macOS Yonder并复验QQ音乐多步骤链路
- [ ] 建立并通过独立Verification Goal
- [ ] Windows对等验证（依用户决定暂缓）

2026-10-01正式验证返回Apply：包边界与隔离Worker通过，但正式Yonder在`list_apps`前的Sky native-pipe握手被服务关闭，阶段为`transport-closed`。同版本Codex CUA REPL可用，说明能力受ChatGPT/Codex可信宿主通道约束；在取得OpenAI支持的外部Broker/授权接线前不得把本Change合入dev或宣称产品链路通过。
