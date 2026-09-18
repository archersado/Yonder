# 历史记录（只读追溯，不作为当前规划）

# E0-S3 ego-lite BUA 集成验证

## Story

作为 Yonder 开发团队，我们需要直接复用 ego-lite Browser Task Space，并把其任务引用、所有权和事件映射为 Yonder Task Status，而不复制 Task Space。

## 调研结论

截至 2026-09-10，ego-lite 官方 README 明确仅支持 macOS，Windows/Linux 仍在 roadmap；官方安装脚本也仅支持 macOS。当前 Windows 环境不存在 `ego-browser` CLI，因而无法形成真实运行证据。

## 范围决定

本 Story 状态为 Deferred。Windows 首版不宣称 BUA；不自行移植 ego-lite，不用 Playwright、Selenium 或 CUA 冒充 BUA。macOS 恢复到产品范围或 ego-lite 发布 Windows Runtime 时重新开启。

OpenSpec：`openspec/changes/e0-defer-ego-lite-bua/`
