# DS-S1 Windows 真实 BUA 展示验证（2026-10-29）

## 结论

独立子 Goal 通过。Windows 11 原生桌面宿主经 Named Pipe Gateway 1.19 和真实 ego-lite Browser Task Space，依次展示 `idle → listening → executing → success → idle`。`browser.execute(create)` 与 `browser.execute(finish)` 的截图均在对应请求尚未返回时取得；真实 `finish` 把任务置为 `completed` 并关闭 `ego:3`，随后展示完整举爪成功姿态，约 2 秒后恢复 idle。全程未执行 CUA。

本结论只关闭上述 BUA 成功链路。`waiting_for_user`、`paused`、`thinking`、`recording` 和真实 `failed` 的 Windows 原生来源仍未在本轮验证；不得据此把九态完整链路、DS-S1 总 Goal 或双平台门禁标记完成。

后续非 CUA 边界探测确认：正式 `task.fail` 被 Gateway 以 `-32010` 拒绝，原因是“任务失败终结需要协议1.18及CUA Runtime”，见 `native-failed-no-cua-blocked-result.json`；已创建的探测任务随后通过正式 `task.cancel` 变为 `cancelled`，测试 Agent 已停用。waiting/paused 隔离夹具在 Gateway 握手前因 Windows Tauri 数据目录不接受 APPDATA 覆盖而拒绝，未创建任务或 ego 空间，见 `native-waiting-paused-result.json`。本轮不为取得截图绕过 CUA Runtime，也不在日常数据库留下无法恢复的非终态任务。

## 首次失败与修复

- 首次真实链路已完成任务，但验证结果漏收两张 executing 截图，且正式内置动画清单缺少 `success/failed`，真实 `success` 因安全回退显示为 idle。原结果保留在 `apps/desktop/evidence/ds-s1-windows-20261029/native-bua-presentation-aggregation-failure-result.json`。
- 依据 Story 已审定的完整角色终态设计，正式清单接入既有完整成功主体、原始完整失败姿态的运行时副本和对应独立闭眼层；未引用已被用户否决的 `success-gesture.png`、`failed-body.png` 或 `failed-head.png` 局部分层素材。既有 1.8 秒终态计时、状态源、协议、持久化均未改变。
- 验证脚本将 create/finish 两张执行期截图纳入结构化结果，并要求两次请求均仍在途、六张截图齐全、成功截图与前后 idle 不同。

## 原生证据

- 结果：`apps/desktop/evidence/ds-s1-windows-20261029/native-bua-presentation-result.json`，`passed=true`、六张截图 SHA-256 全部不同、`cua_executed=false`。
- 真实任务：`task_92eb48a501073774806cd007db07713f`；最终 `completed`。
- Browser Task Space：`ego:3`；`reference_finished=true`，已正常关闭。
- 两次执行期：`executing_create_request_in_flight=true`、`executing_finish_request_in_flight=true`。
- 截图：窗口均为 200×200 逻辑像素；144 DPI 下为 300×300 物理像素。`presentation-success.png` 可见完整举爪展翼角色，与 `presentation-idle-before.png`、`presentation-idle-after.png` 明显可区分；`presentation-executing.png` 可见双爪忙碌姿态。
- 测试 Agent `ds-s1-win-c1c83448d270` 已禁用，归属 `yonder-desktop` 进程已结束，无测试宿主遗留。

## 回归与门禁

- Windows `cargo test -p yonder-desktop --locked`：库 8 项、桌面入口 4 项，共 12 项通过，0 项失败；仅有既有未使用导入与 OpenSSL PDB 链接警告。
- `python scripts/check_architecture.py`：通过。
- `openspec validate e0-validate-desktop-foundation --strict`：通过。
- PowerShell AST 语法检查与相关文件 `git diff --check`：通过。
