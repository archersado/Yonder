---
title: 'Windows SkyDriver 隔离生效验证'
type: 'chore'
ticket: 'CU-S2'
created: '2026-11-04'
status: 'built'
baseline_revision: 'a981dfb0f4dcfde6739e6040975f51e2383b0964'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'thorough'
review_source: 'auto'
lenses_ran: ['edge-case-hunter', 'verification-gap']
review_loop_iteration: 0
context:
  - 'docs/specs/epic-CU/story-CU-S2/product-requirements.md'
  - 'docs/specs/epic-CU/story-CU-S2/architecture-design.md'
  - '_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-01-SDK-ONLY-INTEGRATION.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前机器安装了 `@oai/sky@0.7.13` 和 OpenAI 签名的 Windows helper，但 Yonder 当前主干没有 Windows Sky 产品接线；仅凭文件存在不能判断 SkyDriver 是否能实际 Observe 和执行动作。

**Approach:** 在不修改 Yonder 产品代码的前提下，用官方 Sky JavaScript API 对独立启动的未保存 Notepad 实例执行一次“发现 → Observe → 固定标记输入 → 再 Observe”，输出不含窗口正文和截图的布尔化证据，并清理本次创建的进程。

## Boundaries & Constraints

**Always:** 固定使用本机已安装的 OpenAI 签名发行物及精确版本；目标只允许本次探针创建并由 PID/窗口身份唯一绑定的 Notepad；动作前后均 Observe；证据只记录版本、哈希、阶段、计数和通过/失败布尔值；失败即停止且不重放动作。

**Never:** 不安装或升级依赖，不把 Sky 接入 Yonder Gateway/TaskHost，不修改已接受架构决定，不保存测试文件，不读取或输出其他窗口正文/截图，不操作已有 Notepad，不提升权限，不终止无关 Codex/CUA 进程，不据本次机器验证宣称 Windows 产品链路已完成。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 基础传输 | 固定 Sky 包、helper 和 Codex 私有 PATH | `list_apps` 返回 `target=windows` 且 helper 正常退出 | 启动/协议失败即 FAIL，不切换实现 |
| 隔离动作 | 探针新建的唯一 Notepad 窗口 | 动作前可 Observe；仅输入固定标记；动作后 Observe 可确认标记 | 身份不唯一、Observe 不可用或输入不可确认即停止，不重放 |
| 清理 | 未保存的探针 Notepad 与本次 helper | 只关闭本次创建对象，且不产生保存文件/遗留 helper | 无法安全确认归属时保留现场并报告，不强杀 |
| 权限阻断 | helper、Codex app-server 或目标受 Windows 权限限制 | 返回明确阶段与失败分类 | 不提权、不绕过 UIPI/系统授权 |

</frozen-after-approval>

## Code Map

- `apps/desktop/src/lib.rs` -- 证明当前 Windows TaskHost 未装配 CUA，避免把隔离 Sky PASS 误报为产品 PASS。
- `crates/adapters/src/cua.rs` -- 现有 Yonder 动作后 Observe、unknown 和不重试契约参照。
- `spikes/cua-driver-comparison/RESULTS.md` -- 既有 Windows trycua 同类样本边界与证据格式参照。
- 本机 Codex `runtimes/cua_node/.../node_modules/@oai/sky` -- 本次只读验证对象，不复制进仓库。

## Tasks & Acceptance

**Execution:**
- [x] 固定本机 Sky/helper/`codex.exe` 身份并验证签名、版本、哈希和启动依赖，只保留脱敏摘要。
- [x] 运行基础 `list_apps` 探测，验证 Windows helper 传输、协议响应和正常关闭。
- [x] 运行隔离 Notepad 动作样本，校验唯一窗口、动作前 Observe、一次固定标记输入和动作后 Observe。结果：文本型前置 Observe 为空时输入调用为 0；补充截图型 Observe 后只输入一次，动作返回且截图变化，但后置 AX 仍为空、无法确认固定标记，结果保持 FAIL/unknown 且不重试。
- [x] 清理本次创建的 Notepad/helper，检查未保存文件和遗留进程；汇总为“运行时 PASS/FAIL”与“Yonder 产品链路 PASS/FAIL”两个独立结论。

**Acceptance Criteria:**
- Given 本机固定 Sky 发行物和非提权 Windows 会话，when 完成隔离样本，then 只有在身份唯一、动作前后 Observe 均成功且固定标记被后置事实确认时，Sky Windows 运行时才为 PASS。
- Given 当前 Yonder Windows TaskHost 未接线，when Sky 隔离样本通过，then 产品链路仍明确为 NOT IMPLEMENTED，不得公布 `computer.execute=available`。
- Given 任一阶段出现超时、断连、权限拒绝或不可确认结果，when 汇总验证，then 结果包含稳定失败阶段且动作不重放。

## Implementation Notes

- 2026-11-04：固定发行物为 `@oai/sky@0.7.13`；Windows helper Authenticode 状态为 `Valid`，签名者匹配 OpenAI OpCo, LLC，SHA-256 为 `49C7F43DA0C2A9DBCADFB0C993058147018BF7C1D0FDE4360DE11ACE2C98F45D`，同发行物 `codex.exe` 存在。
- 基础传输 PASS：补齐 Codex 私有目录到子进程 PATH 后，官方 API 返回 `target=windows`、41 个应用条目并正常关闭 helper。未记录应用名、标题、正文或截图。
- 隔离 Notepad 样本 FAIL（`pre_observe`）：两次均通过窗口集合差集得到唯一新窗口；第二次在精确应用授权与 `activate_window` 成功后复核，`get_window_state(include_screenshot=false, include_text=true)` 仍没有 AX tree。两次均在输入前停止，`type_text_calls=0`，没有动作重放或假报成功。
- 清理 PARTIAL：第一次失败后探针只凭“运行前 Notepad 为 0、运行后恰好为 1”终止唯一进程，没有保存启动 PID/句柄；最终进程数虽为 0，但存在快照后用户新开 Notepad 时误杀的竞争窗口，不能把这一步记为严格 PASS。截图型动作样本则始终使用同一 Sky `window.id` 撤销和关闭。
- 结论拆分：Sky Windows helper 的发现/传输运行时为 PASS；计划要求的 Notepad“前置 Observe → 输入 → 后置 Observe”链路为 FAIL，阻断点是前置 AX 为空；Yonder Windows 产品链路仍为 NOT IMPLEMENTED，未修改产品代码或能力声明。
- 截图型 Observe 补充：在 `sky.target` 已初始化为 Windows Client 后安装空的 `emitImage` 接收器；前置 `get_window_state(include_screenshot=true, include_text=true)` 返回 1 张窗口截图、窗口身份一致，图片仅在内存中计算，不输出、不落盘。AX tree 仍为空。
- 动作与后置事实：固定标记的 `type_text` 只调用 1 次并正常返回；后置 Observe 返回 1 张同窗口截图，图片指纹相对前置发生变化，2 次图片发射均被空接收器消费。但后置 AX tree 仍为空，固定标记无法由 Sky 文本事实确认，因此阶段为 `post_observe`、结果为 FAIL/unknown，未再次输入重试。
- 本次截图型样本清理：对同一窗口调用撤销后关闭，`window_closed=true`、`helper_closed=true`；最终 Notepad/helper 进程数均为 0。因 AX 为空且撤销后未取截图，先前的 `cleanup_marker_removed=true` 是由空文本误算出的无效证据；只能证明撤销调用返回和窗口关闭，不能证明固定标记已从现代 Notepad 的会话恢复状态移除。为避免读取用户内容，本次不重新打开 Notepad 检查。
- 权限/环境阻断覆盖：在不把 Codex 私有目录加入 PATH 时，纯只读 `list_apps` 稳定失败于 `helper_launch`，分类匹配 `codex app-server: program not found`；`action_calls=0`、`failed_closed=true`、`helper_closed=true`，未提权或切换实现。
- 修订后的运行时结论：Windows Sky 的 helper 传输、窗口绑定、截图 Observe 与一次 `type_text` 派发均可运行，但现代 Notepad 没有返回 AX 文本，动作后无法证明固定标记，故完整受监管执行链不能判 PASS；Yonder 产品链路结论不变。
- 控制主体 UX FAIL：真实动作期间系统提示显示 ChatGPT/Codex 正在控制，而不是 Yonder。固定 helper 二进制内含 `Codex is using your computer` 与 `Esc to cancel` 文案，Sky JavaScript API/Windows 文档没有调用方品牌参数；helper 还必须启动 `codex app-server`。因此该提示来自 OpenAI 签名 helper 的固定宿主身份，不是 Yonder 当前可合法改名的界面。不得修改或重签 helper；若 Windows 产品要求显示 Yonder，必须由后续 Spike 验证上游正式品牌接口，或采用 Yonder 自有签名的 Windows Driver/控制提示实现。
- 输入目标保护缺口：动作使用同一 Sky `Window` 对象，且前置 Observe 的 `window.id` 一致，但 `type_text` 紧前没有再次读取原生前台 HWND/PID，也没有 Windows 用户输入中断监听；因此不能证明用户在该瞬间切换焦点时绝不会误输入，完整链路继续保持 FAIL/unknown。
- 运行身份补证：当前 Windows 令牌为非管理员、Medium Mandatory Level，满足本次非提权样本前提。
- `codex.exe` 补证：解析路径为 Codex 私有目录 `bin/22cfb1d302613eae/codex.exe`，`codex-cli 0.166.0-alpha.5.1`，OpenAI Authenticode 签名有效，SHA-256 为 `3097C909FB6D5CD63510D44ACEB1D42DF9F6E85722A3E447E477DEF1ECF60E16`。

## Plan Change Log

- 2026-11-04：用户在真实动作样本中明确要求控制提示主体必须为 Yonder。验证结果新增独立 `controller_identity` 失败项；保持本次“仅验证、不接产品”边界，避免把 OpenAI helper 的 ChatGPT/Codex 品牌错误归因为可配置项或通过修改签名二进制绕过。

## Review Triage Log

- `medium / patch`：早期清理只依赖前后进程数量，存在快照后用户新开 Notepad 时误杀的竞争窗口；已把清理结论从 PASS 更正为 PARTIAL，并明确后续探针必须保存启动 PID/句柄，无法确认归属时保留现场。
- `medium / patch`：输入前未原生复核前台 HWND/PID，也没有用户输入中断监听；Sky `Window` 身份只能缩小风险，不能消除焦点漂移，已作为完整执行链不能 PASS 的明确缺口记录。
- `medium / patch`：`cleanup_marker_removed=true` 来自空 AX 文本，不能证明撤销生效；已撤销该证据并披露现代 Notepad 会话恢复中是否残留标记为未知，不读取用户内容做追查。
- `false / reject`：非提权前提不是未验证；复核令牌为非管理员且完整性级别为 Medium。
- `medium / patch`：原记录只固定了 helper 身份，未固定实际 `codex app-server` 发行物；已补齐解析路径、CLI 版本、OpenAI 有效签名和 SHA-256。

## Design Notes

本次验证区分两个问题：已安装 Sky Windows runtime 是否具备真实能力，以及 Yonder 是否已经把该能力接入产品。前者可通过隔离样本回答；后者由当前编译条件可直接判定为否。只有后续明确提出采用 Sky 的 Windows 产品路线时，才进入独立 Story、Spike、ADR、OpenSpec 和双平台验证。

## Verification

**Commands:**
- OpenAI 签名、SHA-256 与精确包版本检查 -- expected: helper 签名有效且版本/哈希固定。
- Sky `list_apps` 脱敏探测 -- expected: `target=windows`、条目数大于零、正常退出。
- Sky Notepad 隔离探针 -- expected: 唯一窗口、前置 Observe、单次输入、后置 Observe、清理均为 PASS，输出不含正文或截图。
- `git diff --check` -- expected: 计划/证据文本无格式错误。

**Results:**
- 发行物身份检查：PASS。
- Sky `list_apps` 脱敏探测：PASS。
- Sky Notepad 文本型前置探针：FAIL at `pre_observe`；输入调用 0 次。
- Sky Notepad 截图型探针：前置截图 Observe PASS；固定标记输入调用恰好 1 次且返回，后置截图变化；因后置 AX 为空、标记不可确认，FAIL/unknown at `post_observe`，未重试。
- 缺失 Codex PATH 阻断：PASS（预期失败关闭）；`helper_launch` / dependency missing，动作调用 0 次。
- 清理检查：PARTIAL；Notepad/helper 遗留进程均为 0，但早期样本存在按数量清理的归属竞争，且截图型样本无法证明撤销后的标记未进入会话恢复状态。
- 输入目标保护：FAIL/unknown；动作使用同一 Sky Window，但输入前未复核原生前台 HWND/PID，也没有用户输入中断监听。
- 非提权运行身份：PASS；非管理员、Medium Mandatory Level。
- `codex.exe` 发行物身份：PASS；路径、`0.166.0-alpha.5.1`、OpenAI 签名和 SHA-256 已固定。
- 控制主体身份检查：FAIL；实际提示为 ChatGPT/Codex，不是 Yonder，且公开 Windows API 未提供品牌覆盖参数。
- Yonder Windows 产品接线检查：NOT IMPLEMENTED；`apps/desktop/src/lib.rs` 的 CUA 组合根仍仅在 macOS 编译。
