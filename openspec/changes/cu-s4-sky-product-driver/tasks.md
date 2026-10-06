# Tasks

- [x] 接受AD-CU-09并同步CU-S4三份设计
- [x] 将组合根、Adapter与Worker切为Sky单栈
- [x] 移除trycua生产依赖、打包资源与环境回退
- [x] 覆盖应用绑定、transcript解析、语义动作与包边界测试
- [x] 以协议1.41接入受限自绘控件双击并完成正式QQ音乐样本
- [x] 构建正式macOS Yonder并复验QQ音乐多步骤链路
- [x] 建立并通过独立Verification Goal
- [x] 外置同任务窗口绑定到Rust Adapter并覆盖Worker重建/跨任务回归
- [ ] 修复Sky坐标点击超时误报、接入新鲜AX元素激活并完成企业微信入口回归
- [ ] Windows对等验证（依用户决定暂缓）

2026-10-01正式验证返回Apply：包边界与隔离Worker通过，但正式Yonder在`list_apps`前的Sky native-pipe握手被服务关闭，阶段为`transport-closed`。2026-10-04进一步确认固定Sky发行物提供官方签名MCP Client，且Client要求OpenAI Team ID父进程；同发行物签名Node派生Client的`list_apps/get_app_state`已通过隔离探针。产品实现改走该签名MCP桥，仍须完成正式Gateway与独立Goal后才能合入dev。

2026-10-04复核：签名MCP桥、身份审计和隔离Worker已通过；正式Gateway进入Sky后因macOS当前锁屏返回`target-window-unavailable`并安全交回，任务已取消。解锁后仍须完成QQ音乐正向与视觉交回样本；此前本Change继续Apply。

2026-10-06 macOS独立Goal PASS：正式Gateway任务完成搜索输入、`ARROWDOWN→Down`候选导航、搜索结果页进入与视觉双击播放；后置截图显示底部播放器为`One Last Kiss - 宇多田光`且处于播放态。终结路由修复后，重建包任务`task_343827d8b795ca1bf50528181e6b9cc0`从正式Gateway落为`completed@23`。Windows对等验证继续按主人决定暂缓，因此Change不Archive。

同日企业微信企业入口初次回归返回Apply：任务`task_080739ed85eebc668da4b97ffcf6bc96`虽返回`fragment-complete@17`，但正式Yonder动作后Observation证明企业菜单未打开；此前点击目标还错误指向底部图标。以正确顶部头像坐标重试的`task_2b366150d400bcd80c2b95ea54d7ef88`仍未打开菜单，却被截图指纹变化误判为完成。该样本暴露截图指纹误报和计划协议不能提交新鲜AX元素索引两个缺口，不计入PASS；协议1.42修复与正确入口回归完成前本项保持未完成。
