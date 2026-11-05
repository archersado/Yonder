# Reviewer Gate：现实与版本核验

审阅日期：2026-11-05
审阅范围：`agent-desktop-driver-spike.md`、AD-CU-07、CU-S4 三份设计、OpenSpec `cu-s4-agent-desktop-driver-spike` 及 2026-11-04 独立研究。
方法：只采用仓库已落盘研究及其指向的 v1.1.0 官方文档、源码/API、Release/Issue 证据；未把 `main` 当前状态或未运行样本当作已证明事实。

## Verdict

**有条件通过进入 Spike，当前不应把架构基底标记为 final 或把 CU-S4 推进到可实施状态。** 候选定位、C ABI 隔离、machine-wide lease 的有限范围、在途取消为 `unknown`、CDP 禁用、Windows CRT 与平台签名风险均与现有证据一致；但统一样本漏掉仓库强制的 Office/WPS/文件管理器场景，且最终集成入口和源码 revision 尚未真正冻结。

## High

### H1：双平台样本漏掉仓库强制的 Office/WPS/文件管理器任务集

- **证据：** 仓库 `AGENTS.md` 明确要求“CUA Driver 升级运行固定 Office/WPS/文件管理器任务集”。现有 AD-CU-07、Story 架构和 OpenSpec 只冻结 Notepad/TextEdit、Electron、身份漂移、DPI、多显示器等样本；没有 Office/WPS 或文件管理器。研究的 Open questions 也明确把 Office/WPS 列为尚未验证的目标应用。
- **影响：** 即使当前矩阵全部通过，也不能覆盖 Yonder 的 Driver 升级完成定义；特别是 Windows 高完整性 WPS、资源管理器 ref/Observe 和 macOS Finder/Office 是现有路线的真实风险区。
- **处理：** 在 AD-CU-07、CU-S4 AC、设计矩阵和 OpenSpec delta 中增加固定 Office/WPS/文件管理器样本。若受权限或许可证限制无法执行，结果应是明确的硬门槛失败/unsupported 证据，而不是静默移除样本。

### H2：`v1.1.0 或唯一修复 commit` 与 `当前仓库 revision` 仍不是可复现的最终版本钉死

- **证据：** 研究验证的是 `v1.1.0`；CRT 修复在 2026-11-04 访问时仍未合入。Spine Stack 同时允许“若阻断则首次执行前冻结唯一修复 commit”，并把 harness 写为“当前仓库 revision”。这在 draft 阶段可以作为前置门禁，但不能作为 final Stack。
- **影响：** 两个实现者可以分别选择 tag、PR head 或后续 main commit，也可以在不同 Yonder commit 上运行，产生不可比较的双平台证据。
- **处理：** 开跑前把两项都改成不可变 SHA：上游源码 commit、Yonder harness commit，并记录 Cargo.lock/制品哈希。若选择 CRT 修复 commit，先重新核查该 commit 是否只包含预期修复及其许可证/供应链来源；两个平台必须完全一致。

## Medium

### M1：`C ABI 或最小桥` 尚未固定唯一入口，结果语义存在分叉

- **证据：** v1.1.0 完整 JSON envelope 有 delivery/retry disposition，但研究已确认部分 C ABI status-only 调用不暴露完整 disposition；公开 C ABI 也未发现 cooperative cancel。当前设计允许“C ABI 或最小桥”，并允许“最小 fork/桥后重验”。
- **影响：** 两个平台或两名实现者可能选择不同入口，导致一方保留 `delivered_unverified`、另一方只能返回 status；这样无法证明 ADS-03/05 的同一结果语义。
- **处理：** 在实现前冻结一个跨平台入口契约：具体导出函数/桥命令、请求/响应 schema、完整 disposition 映射和版本哈希。若必须 fork，记录最小 diff、上游基线、构建命令与许可通知；若无法在两个平台保留同一 envelope，则按淘汰处理。

### M2：trace 与 screenshots 的默认值表述混在一起

- **证据：** 独立研究的最终收窄结论是：结构化 trace 默认开启且部分字段脱敏；screenshots 默认关闭，显式开启后图片不脱敏。现有 ADR/Story 多处写“禁用……默认 trace/screenshots”或“默认敏感 trace”，容易被解释为二者都默认开启。
- **影响：** 可能让实现只关闭截图、漏关默认 trace，或在评审时错误描述上游隐私默认值。
- **处理：** 统一改为“显式关闭默认开启的结构化 trace；保持 screenshots 禁用；不得写入 notification/snapshot/clipboard/command output 等敏感字段”。

### M3：平台签名结论需要区分来源证明、OS 签名和 Yonder 最终签名主体

- **证据：** v1.1.0 Release 有 GitHub/Sigstore provenance；Windows PE 无 Authenticode；公开 macOS 发布流程未见签名/公证，研究核验制品未建立 Apple 平台身份。架构目前把这部分正确地放在 Spike 门槛，但“完成签名/公证责任链”还没有指定证据格式。
- **影响：** 仅验证 GitHub attestation 或临时 ad-hoc 签名，可能被误当作 Windows/macOS 产品身份完成。
- **处理：** 证据矩阵分别记录：源码/构建 provenance、Windows Authenticode signer 与校验结果、macOS Developer ID signer/hardened runtime/notarization/stapling/Gatekeeper 结果。没有正式证书时只能证明“构建可签”，不能宣称责任链已完成。

## Verified / No finding

- **模型无关与平台结构：** Rust core、Windows UIA、macOS AX、CLI/C ABI 的候选定位有 v1.1.0 官方证据，作为低层 Driver 而非 Agent 的表述成立。
- **machine-wide lease：** 文档已正确收窄为参与同一机制的单次 dispatch 互斥，没有误写成任务级 ownership、用户接管或 cancel。
- **取消：** “公开接口没有可依赖的协作式取消；在途 kill 只能 `unknown`”与当前公开 C ABI 的缺席性证据一致，且没有声称物理副作用可回滚。
- **CDP：** 禁止 `launch --cdp` 与本地 HTTP/WebSocket/TCP 符合 v1.1.0 官方命令文档及 Yonder 边界。
- **Windows CRT：** 文档没有把 x64 推断写成直接复现；把干净 Windows 和 CRT 作为硬门槛是保守且证据一致的做法。
- **平台范围：** Windows 10 1809+/Windows 11 验证、macOS 13+ 与研究中的官方平台范围一致；安全桌面/UAC/锁屏仍应保持不支持，不得通过提权扩围。

## Gate 建议

修复 H1，并在真正开跑前关闭 H2/M1；M2 可直接文字修复，M3 写入证据模板。完成后可把 CU-S4 从 `design-review` 推进为“Spike ready”，但 AD-CU-07 仍应保持 Proposed，直到双平台原生证据和独立 Verification Goal 完成。

## 2026-11-05 复核

**结论：PASS（架构/设计 Gate）。** H1、M1、M2、M3 已关闭；H2 的上游 revision 已关闭，harness revision 已转化为明确的首次原生运行前执行门禁，不再存在实现者可自由选择上游版本或入口的架构分叉。

| 原发现 | 复核结果 | 当前证据 |
|---|---|---|
| H1 Office/WPS/文件管理器缺失 | 已关闭 | AD-CU-07、产品需求、Story 架构、OpenSpec spec 与 `sample-catalog.md` 已冻结 Windows WPS/文件资源管理器和 macOS Office/Finder；环境缺失不得判通过，已安装但不支持触发淘汰。 |
| H2 revision 未钉死 | 设计层关闭；保留执行门禁 | 上游固定为 v1.1.0 commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`；2026-11-05 用远端 tag ref 复核一致。禁止因 CRT 漂移源码。harness 尚未实现，故其 commit/dirty diff、依赖锁、toolchain、features、targets、链接策略与制品哈希被明确列为“首次原生运行前”未完成任务；未写入 build manifest 前不得运行证据样本。 |
| M1 C ABI/最小桥分叉 | 已关闭 | 唯一入口固定为 Worker 内最小 Rust bridge + inherited stdio JSONL；明确禁止 status-only C ABI、CLI、其他 IPC 及平台各自入口；evidence schema 固定 disposition 枚举。 |
| M2 trace/screenshots 默认值混淆 | 已关闭 | ADR、Story 与 OpenSpec 已统一为“显式关闭默认开启的结构化 trace，保持 screenshots 禁用”，并排除 notification/snapshot/clipboard/command output 敏感字段。 |
| M3 签名证据未分层 | 已关闭 | OpenSpec 已分别要求 build provenance、Windows Authenticode signer/校验/策略结果，以及 macOS Developer ID、hardened runtime、notarization、stapling、Gatekeeper；明确测试证书、ad-hoc 签名和 Sigstore provenance 不能冒充生产平台身份。 |

### 残留执行门禁（非架构阻断）

- 首次原生运行前必须完成 `tasks.md` 中未勾选的 build manifest 冻结项；否则本 PASS 自动失效，不能接受任何 Windows/macOS 样本证据。
- AD-CU-07 必须保持 Proposed，直至双平台硬门槛、签名/干净机证据和非实现者 Verification Goal 全部通过。本 PASS 不授权产品接线或替换 trycua。
