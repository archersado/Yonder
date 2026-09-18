# Verification Goal：Agent Browser Gateway

状态：macOS通过；Windows按用户决定暂缓。日期：2026-09-16。

独立验证执行正式预览应用、本地UDS Gateway及ego-lite SDK：协议协商为1.8，Agent依次登记任务、声明创建步骤、执行并Observe `ego:43`、推进已观察边界、声明结束步骤并finish。任务最终为`completed`、sequence为10，10条事件连续且包含2条observed attempt；finish后外部Task Space已关闭。

自动检查：`cargo run -p yonder-protocol --example generate --locked -- --check`与`cargo test --workspace --locked`通过。真实证据见`apps/desktop/evidence/browser-gateway-20260916/result.json`。首次验证发现stopped attempt历史结果读取失败，已在统一SQLite读取处修复并加入推进后读取回归检查。
