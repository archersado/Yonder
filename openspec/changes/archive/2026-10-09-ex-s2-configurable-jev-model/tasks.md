# 任务

- [x] 更新 AD-EX-03 与 EX-S2 三份设计，明确本地模式和模型配置边界
- [x] 扩展 `JevConfig` 校验、既有 JSON 兼容与 SQLite 回归
- [x] 将配置模型传入候选选择/步骤核验 Worker，移除 `jev-latest` 硬编码
- [x] 扩展独立 Jev 设置窗口的模型字段与校验
- [x] 覆盖 Application、Adapter、桌面 UI 测试并严格校验 OpenSpec
- [x] 配置化置信度字段与阈值，并兼容历史 `confidence + 0.75`
- [x] 设置面板支持置信策略，Worker 严格读取选定字段且不静默回退
- [x] 覆盖字段缺失、非法数值、阈值门禁与 UI 校验回归
- [x] 将置信字段枚举改为可填写字段名，留空默认 `confidence`，并验证旧 `confidence_metric` 迁移
- [x] 重建正式 macOS `Yonda.app`，通过 Gateway 运行本地 Jev Verification Goal
- [x] 审阅、归档并 ff-only 合并 `dev`
