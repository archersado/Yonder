# DS-S1 正常应用启动计时 Verification Goal

日期：2026-09-14；Story：DS-S1；Change：e0-validate-desktop-foundation。状态：当前样本未通过3秒门槛，不Archive。

后续用户明确放宽当前阶段性能并要求该功能pass，已先更新AD-E0-01及架构主干、Story、delta spec。依新判定，本机正常应用启动渲染就绪功能 **PASS**；4.31秒读数保留，3秒转为优化目标。原始非零退出是旧门槛结果，不重写历史JSON。完整交互与整项Story仍分别验收。

现有启动脚本直接执行.app内二进制，不能覆盖LaunchServices路径。本轮仅修正验证脚本为`open -n --stdout … --stderr … Yonda.app`，未修改产品代码。复用既有固定渲染报告，检查图片解码、600像素运行时素材、样式与脚本就绪；日志不包含用户内容。

通过既有托盘入口正常退出PID 56576后，执行：

```text
python3 spikes/desktop-foundation/measure-startup-macos.py spikes/desktop-foundation/evidence/launchservices-startup-20260914.json
```

证据为上述JSON与同名.log。新宿主PID 61042，SHA-256为`9ee83d06b71615125aac0b208559ed64a3ce8d9ca8db787df2badd7574988402`。渲染检查ready=true，启动至报告4314.508333毫秒；脚本因超过3000毫秒以非零状态退出，保留应用运行与失败证据。

计时包含LaunchServices命令耗时、既有页面加载后300毫秒诊断等待及轮询/进程查询耗时；不是冷缓存启动、首帧或完整交互就绪。因此本样本不能通过启动门槛，也不能仅凭该数值断言首帧必然超过3秒。下一步需区分应用实际就绪时刻与测量开销，再定位启动耗时；不得以重复取最快样本代替失败记录。

`git diff --check`通过。Windows仍由用户暂缓，本验证不改变完整Story或ADR状态。
