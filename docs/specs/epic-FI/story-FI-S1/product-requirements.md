# FI-S1 产品需求

## 问题与目标

绝对路径防穿越，别名不能绕过同文件互斥，删除默认回收站。

## 范围与非目标

本 Story 仅负责“规范文件身份与受控操作”。首批实现 macOS 内部 File Port/Adapter；Windows 按用户决定延期并明确 unavailable。Agent Gateway 与覆盖/删除确认 UI 不在本增量开放。

## 验收条件

- FI-01：目标行为有可复现成功样本，失败不得伪报成功。
- FI-02：输入只接受规范化绝对路径；未创建输出通过既有父目录验证授权根，阻止路径穿越和链接逃逸。
- FI-03：软链接、硬链接、大小写别名和重命名不能绕过同文件写互斥。
- FI-04：写入使用同目录临时文件，校验后原子提交；冲突或失败不留下正式目标。
- FI-05：Office/WPS已打开文件不得被绕过锁覆盖；锁冲突返回可观察等待原因。
- FI-06：删除默认进入系统回收站；永久删除不在MVP默认入口。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

Windows 文件身份与 Office/WPS 锁实机证据、Agent Gateway 文件授权引用及任务事件接线仍待后续增量。

## macOS Runtime 增量（2026-09-25）

来源：FI-01～06、Accepted AD-FI-01 与 macOS/WPS Spike。

- FI-RUNTIME-01：既有文件经规范路径、软链接或硬链接访问时归并到相同 `st_dev + st_ino` 身份；所有路径必须位于规范化授权根内。
- FI-RUNTIME-02：读取最多 16 MiB，并返回稳定身份和 SHA-256；读取过程中身份或大小变化不得返回伪稳定快照。
- FI-RUNTIME-03：新建必须目标不存在；替换必须同时匹配预期身份与 SHA-256。写入在同目录独占临时文件中完成，调用方校验、文件同步、原子替换及父目录同步后才返回成功。
- FI-RUNTIME-04：同文件写租约与宿主 advisory lock 任一冲突均拒绝；失败清理临时文件，提交结果不明不自动重试。
- FI-RUNTIME-05：删除仅可信 LocalUser 可调用，并通过 macOS 系统回收站；Agent 不能自报确认，永久删除没有入口。
- FI-RUNTIME-06：非 macOS 构建只返回 unavailable，不使用路径字符串锁、直接删除或非原子复制兜底。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：目标用户与工作场景、MVP 主干链路。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：Command、File 与 Document。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
