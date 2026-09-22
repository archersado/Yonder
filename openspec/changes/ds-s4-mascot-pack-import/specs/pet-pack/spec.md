# 桌宠资源包导入规范

## ADDED Requirements

### Requirement: 仅导入完整且受限的 v1 资源包

系统 SHALL 校验 ZIP 内仅存在 `manifest.json` 与 `assets/` 下的一级 PNG/WebP 文件，且 manifest 为 `format_version: 1`、恰好九个固定状态。每个状态 SHALL 有 1～16 个唯一帧；ZIP 源不超过 64 MB，manifest 不超过 1 MB，总解压不超过 32 MB，单帧不超过 8 MB 或 1024×1024，总像素不超过 64 MP，条目不超过 145。

#### Scenario：不完整或越界
- **WHEN** 缺失状态、重复路径、非图像条目、损坏图像或任一限额越界
- **THEN** 系统拒绝导入并删除暂存目录

### Requirement: 失败保留当前资源包

系统 SHALL 在应用数据目录内先写入受限暂存目录；全部校验通过后才切换当前包。当前包替换失败或切换回滚失败时，SHALL 保留上一个可用资源包并返回失败。

#### Scenario：切换失败
- **WHEN** 暂存目录无法替换当前包
- **THEN** 系统回滚旧包、删除新暂存内容，且不发布成功反馈
