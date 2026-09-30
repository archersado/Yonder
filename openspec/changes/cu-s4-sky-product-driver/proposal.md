# Proposal：CU-S4 切换 Sky 单一产品 CUA Driver

关联Story CU-S4与Accepted AD-CU-09。Architecture Impact：architecture-change（macOS CUA技术栈、组合根与打包依赖）；不改SQLite或状态所有者。

## Why

正式QQ音乐同实例对照中，trycua无法取得应用内元素，Codex Computer Use / Sky可稳定返回完整应用级AX transcript及元素index。继续补丁式扩展trycua不能形成通用元素优先执行链路。

## What Changes

- macOS组合根仅构造固定Sky Worker，移除trycua生产依赖、Driver开关和回退。
- Worker按任务绑定应用，解析新鲜应用级AX transcript并执行元素index动作。
- 通用计划语义映射到Sky动作，每步后重新Observe；失败交回且不重试。
- 正式包只携带Yonder Worker/Node/Jev，Sky来自受信任外部安装并做固定身份校验。
- 增加Adapter、包边界和正式QQ音乐macOS验证。

## Non-goals

不引入第二模型循环、不持久化AX transcript、不复制Sky包或Computer Use App、不恢复trycua回退、不外推Windows。
