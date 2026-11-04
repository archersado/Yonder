# Verification Goal：Agent Browser Gateway

状态：PASS（macOS 2026-09-16；Windows补证 2026-10-25）。

独立验证执行正式预览应用、本地UDS Gateway及ego-lite SDK：协议协商为1.8，Agent依次登记任务、声明创建步骤、执行并Observe `ego:43`、推进已观察边界、声明结束步骤并finish。任务最终为`completed`、sequence为10，10条事件连续且包含2条observed attempt；finish后外部Task Space已关闭。

自动检查：`cargo run -p yonder-protocol --example generate --locked -- --check`与`cargo test --workspace --locked`通过。真实证据见`apps/desktop/evidence/browser-gateway-20260916/result.json`。首次验证发现stopped attempt历史结果读取失败，已在统一SQLite读取处修复并加入推进后读取回归检查。

Windows补证通过正式当前用户Named Pipe Gateway协商协议1.17，`browser.execute`为available；真实ego-lite空间`ego:8`在同一任务中完成create、hand-off、take-over和finish，4个动作均提交Observed attempt，任务最终为`completed@18`。结构化证据见`apps/desktop/evidence/bu-s2-windows-20261025/result.json`。
