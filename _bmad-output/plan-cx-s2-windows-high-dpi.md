---
title: 'CX-S2 Windows 200% DPI 圈选截图扩围'
type: 'feature'
ticket: ''
created: '2026-11-04'
status: 'built'
baseline_revision: '5fa0651af04a0cfd0890057c849ddc8a4fcd2548'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'thorough'
review_source: 'auto'
lenses_ran: ['blind-hunter', 'edge-case-hunter', 'verification-gap', 'intent-alignment']
review_loop_iteration: 0
context:
  - 'docs/specs/epic-CX/story-CX-S2/'
  - '_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CX-01-EXPLICIT-REGION-CAPTURE.md'
  - 'openspec/changes/cx-s2-region-capture-spike/'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**问题：** Windows 192 DPI（200%）主屏被产品门禁返回 `capture-unsupported-scale`，前端因此隐藏截图。

**方案：** 先用无敏感样本验证真实 192 DPI WGC，再把 Preview 范围从 96 扩至 96/192 DPI，复用既有会话、内存 PNG 与附件链。

## Boundaries & Constraints

**Always：** 按 Spike→AD→Story 设计→OpenSpec→实现→独立验证→Archive；只接受同轴 96/96、192/192 DPI；CSS 坐标只换算一次；像素只在有界内存；失败清旧图并进入无图 `reviewing`。

**Never：** 不开放 125%/150% 或宣称完整 CX-S2；不放宽像素/附件上限；不增加 GDI、协议、持久化、全局监听或状态所有者。跨屏、应用切换、受保护内容、撤权及拔屏仍延期。

## I/O & Edge-Case Matrix

| 场景 | 输入 / 状态 | 预期行为 | 错误处理 |
|---|---|---|---|
| 200% 圈选 | 192 DPI 单屏 CSS 矩形 | 正确物理矩形和 PNG | 不重复乘 DPI |
| 未验证缩放 | 120/144/168 或异轴 DPI | 不返回像素 | 清图并进入仅提问 |
| 超限/捕获失败 | 大选区、超时、帧变化 | 无旧图或伪图 | 保留文字/语音/重选/取消 |
| 旧异步结果 | generation 已变化 | 不覆盖新会话 | 释放资源 |

</frozen-after-approval>

## Code Map

- `spikes/region-capture/` -- 192 DPI 四色探针；用 `GetClientRect + ClientToScreen` 取物理 client rect，避免 WinForms DPI 虚拟化。
- `region_capture_windows.rs::supported_dpi/read_frame` -- 唯一硬拒绝点；WGC 与 `rcMonitor` 已是物理像素，禁止再乘缩放。
- `main.rs::physical_region/region_preview_capture`、`region-preview-feedback.*` -- 单次 CSS→物理映射、generation 隔离与预览/降级投影。
- `AD-CX-01`、`ARCHITECTURE-SPINE`、CX-S2 四份文档、`docs/specs/README.md` -- Spike PASS 后记录 96/192 范围。
- `openspec/changes/cx-s2-windows-high-dpi/` -- 新 proposal/design/tasks/delta spec 和验证目标。

## Tasks & Acceptance

**Execution：**
- [x] `spikes/region-capture/` -- 真实 192 DPI 连续三次验证尺寸、四色、释放与不落盘；失败即停止。
- [x] 架构与 Story 文档 -- Spike PASS 后接受 96/192 DPI，并同步需求来源和剩余门禁。
- [x] `openspec/changes/cx-s2-windows-high-dpi/` -- 建立 conforming 增量规格。
- [x] Rust/前端相关文件 -- 扩围 192 DPI，补坐标、门禁和失败回归，不放宽限额。
- [x] Windows 构建、产品预览、独立 Verification Goal -- 绑定结构化证据与源码/二进制哈希；PASS 后 Archive。

**Acceptance Criteria：**
- Given 192 DPI 四色窗口，when 共享 Rust WGC 连续捕获三次，then 尺寸/四色一致、资源释放且不落盘。
- Given 200% 下 240×160 CSS 选区，when 产品捕获，then 得到 480×320 物理矩形并显示 PNG。
- Given 未验证 DPI、跨屏、超限或 WGC 失败，when 捕获结束，then 无旧图、无重试且可仅提问。
- Given 会话已重选或取消，when 旧结果返回，then generation 拒绝它且不残留图片。

## Implementation Notes

- 2026-11-04：独立C++ Spike在真实192 DPI主屏三连得到480×320四色PASS；随后才扩围AD/Story/OpenSpec和产品门禁。
- 共享Rust探针同时完成192 DPI主屏480×320与96 DPI负坐标副屏240×160各三连PASS；像素未落盘，进程无残留。
- Application 21、Desktop lib 9、Desktop bin 16、前端2项、Windows build、架构检查和OpenSpec strict验证PASS。审查补丁后96/192 DPI探针再次三连PASS；2026-11-04用户在当前200%主屏独立确认图片可见且重选/取消可用，Verification Goal PASS。

## Plan Change Log

## Review Triage Log

| 来源 | 结论/路由 | 证据与处理 |
|---|---|---|
| blind B1 | false | WorkBuddy条目是用户选择拆分后由工作流强制登记的独立延期项，`source_plan: none`符合规定格式。 |
| blind B2 | medium / patch | `working-tree`虽未冒充最终revision，但当前证据不能精确绑定源码/探针；补自动生成的源码与二进制哈希。 |
| blind B3 | medium / patch | 192 DPI C++证据缺生成时间、命令与构建哈希；补可复跑harness和结构化元数据。 |
| blind B4 | false | C++/Rust探针验证WGC物理像素，`physical_region`单测独立验证CSS→物理换算；两层证据没有互相替代。 |
| blind B5 | false | 批准边界是同轴192 DPI能力，不是仅主显示器身份；PMv2/`rcMonitor`路径不按主副屏分支，96 DPI负坐标已覆盖坐标分量。 |
| blind B6 | low / patch | 200%单测缺负原点、小数floor/ceil和work-area偏移组合；补直接回归。 |
| blind B7 | medium / patch | 仅测迟到失败，未直接覆盖迟到成功PNG；提取共用提交守卫并补成功结果回归。 |
| blind B8 | low / patch | Delta spec对跨屏、超限、超时/WGC失败缺独立Scenario；补可验收场景。 |
| blind B9 | false | Verification Goal和任务明确保持待执行、未PASS/未Archive；真实确认卡是当前保留门禁，不存在过度批准。 |
| blind B10 | false | 构建/测试命令可直接复跑且实现报告已给出计数；Goal仍待独立复核，没有把文档断言当最终结论。 |
| edge E1 | medium / patch | 探针定位后未重新确认client仍在原显示器/DPI；运行前重取显示器与DPI并失败关闭。 |
| edge E2 | low / patch | `clear()`不递增generation，虽由phase拒绝旧结果但不满足冻结声明；取消时递增代次并回归。 |
| verification V1 | low / rejected | 192 DPI产品自动E2E确实缺失，但冻结计划要求真实人工检查，历史已将完整Tauri鼠标自动化延期；新增完整门禁复杂度超过本增量，人工检查继续阻断Goal。 |
| intent I1 | low / rejected | 用户可见产品面仍待人工确认属实；任务与Goal公开保持未完成，因此这是待执行验收而非已交付偏差。 |

## Design Notes

PMv2 下 work area、`rcMonitor` 和 WGC 纹理均为物理像素；只扩展受支持 DPI，不再次乘 2。保留纹理边界检查作为显示变化时的 fail-closed 门禁。

## Verification

**Commands：**
- `powershell -File spikes/region-capture/run-windows.ps1` -- 96/192 DPI 三连样本通过。
- `cargo test --offline --locked -p yonder-application -p yonder-desktop`、`cargo build --offline --locked -p yonder-desktop` -- 分层测试及构建通过。
- `node --test apps/desktop/ui/region-preview-feedback.test.mjs`、`python scripts/check_architecture.py` -- 前端与架构门禁通过。

**Manual checks：**
- 200% 主屏真实圈选检查图形，预览可见且重选/取消可用；只记录结构化结果。
