# CU-S2 受监管桌面执行与 Observe

Story: CU-S2
Epic: CU
Status: verifying
OpenSpec: cu-s2-supervised-dispatch-observe
Work Focus Increment: [cu-s2-native-work-focus](../../../../openspec/changes/cu-s2-native-work-focus/proposal.md)
Agent Gateway Increment: [cu-s2-agent-computer-gateway](../../../../openspec/changes/cu-s2-agent-computer-gateway/proposal.md)
Cross-Space App Focus Increment: [cu-s2-cross-space-app-focus](../../../../openspec/changes/cu-s2-cross-space-app-focus/proposal.md)

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

首批实现已按 Accepted AD-CU-04 限定为 macOS 后台 AX 输入、受监管Worker与动作后强制 Observe。固定版本正式资源、macOS预览打包及Yonda宿主权限责任链已就位；正式发布签名和 Windows 证据仍未齐。

## OpenSpec 与验证

[OpenSpec Change](../../../../openspec/changes/cu-s2-supervised-dispatch-observe/) 与独立 Verification Goal 已建立。

2026-09-15：首批 macOS 后台 AX 输入已通过独立 Verification Goal：真实 prepared attempt 经产品 CU Port 派发，动作后强制 Observe 与原生控件一致，Worker 退出且不启动上游 App。完整 Story 仍等待结果事务、产品打包/宿主权限、停止接管及 Windows 证据。

2026-09-14：新增[执行身份/停止联合设计](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)与三份设计验收补齐，AD保持Proposed；当前产品实现状态不变。下一项完成AG步骤/动作去重、Port/事务/迁移及可信WorkRef原生复核Spike，不因设计文档补齐宣称接管已实现。

2026-09-14工作身份隔离Goal PASS，见[独立验证](../../../../openspec/changes/e0-compare-cua-drivers/verification-work-identity-macos.md)。关闭/替换/重启旧引用拒绝，正常/最小化及新Observe有效；仅技术子范围，下一项AG步骤/动作去重与Rust Port/控制事务/迁移定稿，产品接管尚未启用。

2026-09-16：AD-CU-03接受生产WorkRef Adapter子范围。实现仅包含Rust Port、宿主内保留原生对象与macOS精确定位；不新增任务状态、UI成功提示或Recording。

生产WorkRef Adapter子范围PASS：真实macOS窗口正常前置、最小化恢复、几何保持、诱饵/关闭/释放拒绝均通过；见[独立 Verification Goal](../../../../openspec/changes/cu-s2-native-work-focus/verification-goal.md)。下一增量由TM-S3持久化定位阶段并在停止确认后编排调用。

2026-09-16：Accepted AD-CU-05允许把已验证的后台AX文本动作接入Agent Gateway；目标由Yonder解析当前最前方唯一工作窗口，用户输入中断优先，Agent不得提交原生身份或Driver路径。实施与验证见`cu-s2-agent-computer-gateway`。

2026-09-16：Agent Computer Gateway macOS增量PASS。按用户修订升级至协议1.11通用`tool_name + arguments`桥接，动作名称与参数Schema只读取CUA SDK；`type_text`和`press_key`两个真实SDK工具均完成且动作后Observe有效。真实鼠标输入使任务转为`interrupted/unknown-user-input`并释放桌面租约，不自动重试、不启动Recording。证据见[独立 Verification Goal](../../../../openspec/changes/cu-s2-agent-computer-gateway/verification-goal.md)；Windows按用户决定暂缓，CU-S2仍保持verifying。

2026-09-16连续动作增量PASS：正常步骤复用同一受监管Node/trycua Driver会话，真实UDS双动作期间Worker PID不变，任务完成后退出；每步Observe、异常终止与不重试语义保留。证据见`apps/desktop/evidence/computer-continuous-session-final-retry-20260916/result.json`。

2026-09-16协议1.12增量PASS：Agent通过单次`computer.step`完成步骤声明、CUA动作、动作后Observe和步骤推进，MCP不再发现底层`computer.execute`。真实UDS连续动作返回有界UI树证据并复用同一Worker，完成后退出；截图仅接受trycua SDK输出并在任务结束清理，当前系统未返回截图时保持可选字段为空，不使用外部截图工具。证据见`apps/desktop/evidence/computer-step-observe-20260916/result.json`。

2026-09-17真实企业微信任务暴露跨Space缺口：SDK后台启动成功不等于用户当前可见，精确`bring_to_front`在App Translocation窗口上拒绝。已建立独立增量，保持SDK动作原义，并为同一任务后续显式前置绑定最近启动结果的可信身份；不得把`launch_app`隐式组合、硬编码Dock坐标或绕过SDK。

2026-09-17跨Space应用前置增量PASS：Worker按SDK已校验bundle id刷新主进程PID，只为同任务显式`bring_to_front`注入可信身份；真实企业微信任务验证后台启动`target_visible=false`、显式前置`target_visible=true`并完成到`completed@10`。证据见[独立 Verification Goal](../../../../openspec/changes/cu-s2-cross-space-app-focus/verification-goal.md)。Windows继续暂缓，完整CU-S2保持verifying。

2026-09-18：已确认的trycua 0.25.0 SDK-only选型同步到正式桌面目录：`apps/desktop/cua`持有唯一运行时依赖清单与锁文件，正式预览打包脚本从该目录组装Node、SDK、平台原生包和`crates/adapters` Worker；产品验收不再从Spike读取运行依赖。正式签名发布与Windows证据仍未完成。

2026-09-18：重新启动正式预览后，真实UDS协议1.12报告`computer.execute=available`，与既有同一正式应用路径的连续动作/Observe成功证据闭合宿主权限责任链。本轮复测期间两次真实用户输入均使动作安全转为`unknown/user-input`且未重试，符合用户输入优先约束。
