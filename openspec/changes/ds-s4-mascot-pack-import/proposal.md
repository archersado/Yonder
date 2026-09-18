# 提案：桌宠动画资源包导入

Story：DS-S4。来源：产品简报 MVP 主干链路步骤1、桌宠；架构主干「Recording 与桌宠」。

变更：桌面宿主导入受限 ZIP，验证声明式 PNG/WebP manifest 后在应用数据目录原子切换当前桌宠资源包。失败保留内置或上一有效资源包。Hatch Pet 式生成仅为 DS-S4/AD-DS-04 的后续设计，本 Change 不实现生成、参考图上传或 Agent 委托。

Architecture Impact：conforming。任务、Gateway、SQLite 与 Agent 协议不变。
