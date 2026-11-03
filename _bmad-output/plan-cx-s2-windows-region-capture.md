---
title: 'CX-S2 Windows 圈选截图'
type: 'feature'
ticket: ''
created: '2026-11-03'
status: 'built'
baseline_revision: 'aa378f6b314c8ff8361f68ee99003b6b79053a42'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'thorough'
review_source: 'auto'
lenses_ran: ['blind-hunter', 'edge-case-hunter', 'verification-gap', 'intent-alignment']
review_loop_iteration: 0
context:
  - 'docs/specs/epic-CX/story-CX-S2/README.md'
  - '_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CX-01-EXPLICIT-REGION-CAPTURE.md'
  - 'openspec/changes/cx-s2-region-capture-spike/'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**问题：** Windows 圈选固定报未实现；捕获失败后会话滞留 `Capturing`，无图提交可能失败。

**方案：** 先用 CX-S2 Spike 验证 Windows Graphics Capture（WGC），再接入现有 PreviewSession/附件链路：捕获显示器首帧，按物理选区裁剪，内存编码 PNG/Base64。

## Boundaries & Constraints

**Always：** 仅显式圈选后捕获；像素只在内存；隐藏选择层后截图；失败进入无图 `Reviewing`；以显示器物理原点处理负坐标；保留 CUA 暂停、限额、超时、清场；证据不含截图。

**Never：** 不用 GDI/BitBlt 兜底，不落盘，不增加全局监听、状态所有者、协议或持久化，不宣称未验证场景通过，不把 CX-S2 标记 Done。

**范围决定（2026-11-03，用户确认）：** 首批只在 100% 缩放显示器启用图片预览；其他缩放降级为无图 `Reviewing`。125%/150%/200% 经实机统一样本验证后再扩围。

## I/O & Edge-Case Matrix

| 场景 | 输入/状态 | 预期行为 | 错误处理 |
|---|---|---|---|
| 主屏/副屏 | 100% 缩放、单显示器内矩形 | PNG 尺寸和位置正确 | 跨屏拒绝并转无图确认 |
| 不支持/失败 | 非 100%、超时、受保护内容 | 不显示伪截图 | 无图 `Reviewing` 可提交 |
| 终止 | Esc、重选、切屏、超时 | 释放资源和图片 | 回到合法状态 |

</frozen-after-approval>

## Code Map

- `openspec/changes/cx-s2-region-capture-spike/`、`spikes/region-capture/`：Windows 无敏感探针；PASS 前不接产品。
- `AD-CX-01-EXPLICIT-REGION-CAPTURE.md`、`docs/specs/epic-CX/story-CX-S2/`：记录 Windows 路线与剩余门禁。
- `apps/desktop/src/main.rs`、新建 `region_capture_windows.rs`、`Cargo.toml`：WGC/D3D11 捕获、裁剪、编码和超时。
- `region_preview.rs`、`region-preview.js`：失败转无图 Reviewing；补正式 `icon.ico`。

## Tasks & Acceptance

**Execution：**
- [x] 扩展 Spike：四色窗口、WGC 探针、脚本、双屏负坐标结构化结果。
- [x] PASS 后更新 AD/Story，并创建 Windows 产品 OpenSpec Change。
- [x] 实现捕获 Adapter、命令接线、失败状态转换及测试；不改协议/存储。
- [x] 构建宿主，完成双屏验证和独立 Verification Goal。

**Acceptance Criteria：**
- 100% 主/副屏有效选区返回位置、尺寸、色块正确且无覆盖层的 PNG。
- 失败时无伪图片，文字提交、重选、取消均合法。
- 终止或显示变化时释放资源和图片，不自动重试。

## Implementation Notes

- Windows真实环境：96 DPI负坐标副屏连续三次PASS；192 DPI主屏按批准边界降级，不纳入截图通过范围。
- Application 20项与Desktop可执行目标10项测试通过；Desktop库另有3项既有固定路径测试在Windows缺文件失败。
- `cargo build --offline --locked -p yonder-desktop`、前端行为/语法和架构关联门禁通过；新版进程已启动。
- 矩阵审计：正常行由WGC四色三连样本覆盖；失败行由192 DPI降级与状态测试覆盖；终止行由PreviewSession清场/重选测试覆盖。

## Plan Change Log

## Review Triage Log

| 来源 | 发现 | 裁决 | 证据/处理 |
|---|---|---|---|
| blind-hunter B1 / edge-case E5 | 光标禁用结果曾被忽略 | fixed | 禁用失败即关闭失败，不捕获可能含光标的帧。 |
| blind-hunter B2 / edge-case E6 | 超时与错误路径未显式释放WGC资源 | fixed | `CaptureResources` RAII移除事件并关闭session/frame pool；帧读取后显式关闭。 |
| blind-hunter B3 | `TryGetNextFrame`可能吞掉任意错误 | fixed | 改用`FrameArrived`事件和同步通道，仅按明确超时失败。 |
| blind-hunter B4 | 负坐标选区曾被静默钳制 | fixed | `physical_region`验证viewport边界并保留负物理原点；越界失败关闭。 |
| blind-hunter B5 | 密码框、隐私窗口、系统安全界面和用户排除应用证据不足 | deferred | WGC受保护像素由系统屏蔽，但统一样本未覆盖普通密码控件与用户排除应用；记录到`_bmad-output/deferred-work.md`。 |
| blind-hunter B6 / edge-case E7 | 受保护内容黑帧可能当成功 | rejected | 黑色也是合法内容，启发式拒绝会误伤；不增加黑帧猜测，保持系统保护边界并以后续专用样本验证。 |
| blind-hunter B7 | 像素释放/不持久化声明证据过硬编码 | fixed | 探针显式释放像素后记录，测量残留进程并补充环境、版本和工作树元数据；模块无文件写入。 |
| blind-hunter B8 / edge-case E9 | PASS未强制覆盖负坐标 | fixed | 脚本要求至少一个负原点样本且连续三次通过。 |
| blind-hunter B9 / edge-case E8 | DPI awareness调用结果未校验 | fixed | 脚本检查调用结果，失败即停止。 |
| blind-hunter B10 | 缺少完整产品E2E | accepted-risk | 产品换算/接线测试、共享模块真实负屏探针和前端可执行行为测试形成分层证据；独立复核判定批准范围充分，完整Tauri鼠标E2E列入延期。 |
| blind-hunter B11 | Spike双平台任务过早完成 | fixed | 恢复完整双平台Spike任务为未完成，仅接受Windows 100% DPI有界路线。 |
| blind-hunter B12 | 证据元数据不足 | fixed | 补充UTC、源码修订、工作树、OS、GPU与驱动信息；源码/二进制哈希仍列为后续增强。 |
| edge-case E1 | 当前显示器失败时回退主屏 | fixed | 移除回退并失败关闭，避免抓错显示器。 |
| edge-case E2 | 旧异步结果可能污染新会话 | fixed | PreviewSession增加generation并校验捕获代次。 |
| edge-case E3 / E12 | 隐藏窗口或进入捕获失败会留下不一致UI | fixed | 所有失败走幂等无图转换；后端已Idle时关闭Preview。 |
| edge-case E4 | 展示预览失败时后端可能仍持有图片 | fixed | `review_without_image`允许从Reviewing清除已有图片，前端随后投影仅提问DOM。 |
| edge-case E10 / intent I6 | 实现丢弃首帧但文档声称首帧 | fixed | 改为使用首个可用帧并统一文档表述。 |
| edge-case E11 | Windows显示拓扑监听为空 | fixed | 会话期每250ms检查位置、尺寸与缩放签名，变化时统一清场。 |
| verification-gap V1 / intent I4 | 产品负坐标换算路径缺证据 | fixed | 正式路径调用`physical_region`；单测覆盖负Y work area与拒绝越界，真实共享WGC负屏三连PASS；独立复核PASS。 |
| verification-gap V2 | 192 DPI在调用共享模块前被脚本跳过 | fixed | 探针实际调用同一`capture_region`并得到`capture-unsupported-scale`且无像素；独立复核PASS。 |
| verification-gap V3 | 前端测试只做源码字符串匹配 | fixed | 改为可执行helper测试，验证后端转换、旧图清除、DOM仅提问及继续提交链路；独立复核PASS。 |
| intent I1 | 完整端到端表面与原生模块证据存在距离 | accepted-risk | 分层证据已覆盖批准范围；完整Tauri鼠标自动化不作为本次阻断项并记录延期。 |
| intent I2 | 成功附件链未新增端到端测试 | rejected | 附件提交路径未变，既有`agent_input`测试与AD-CX-02证据仍适用；本变更测试新增/无图分支。 |
| intent I3 | 失败集成证据分散 | fixed | 后端状态测试与可执行前端行为测试共同覆盖清图、仅提问和提交。 |
| intent I5 | “Windows支持”可能超出有界范围 | rejected | 冻结范围明确由用户批准为仅100% DPI，所有文档均保留该限制且不宣称完整CX-S2完成。 |

独立非实现者复核结论：上述V1、V2、V3均PASS；本Change的有界Verification Goal可通过，但不等于完整CX-S2 Done。

## Design Notes

采用 `CreateForMonitor`、free-threaded frame pool、staging texture Map；裁剪以 `monitor.position()` 为原点，只启用 Spike 通过范围。

## Verification

**Commands：** `cargo test --locked -p yonder-application -p yonder-desktop`；`cargo build --locked -p yonder-desktop`；`powershell -File spikes/region-capture/run-windows.ps1`。

**Manual checks：**
- 主屏/负坐标副屏检查预览、重选、Esc、切屏、显示变化；只存结构化证据。
