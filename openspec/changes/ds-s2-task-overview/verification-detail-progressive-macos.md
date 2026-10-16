# DS-S2 任务详情渐进展示独立 Verification Goal

日期：2026-10-16  
平台：macOS 26.5，arm64  
范围：DETAIL-PERF-01～04；不替代 DS-S2 Windows 原生证据门禁。

## Goal

证明核心任务详情不再被时间线、浏览器关联、文件授权、命令批准逐项预览或产物清单阻塞；附属失败保持局部，切换/刷新仍丢弃迟到结果，页面不读取或展示完整 Agent Payload。

## 隔离样本与结论

使用 `scripts/verification/desktop/check-task-space.mjs` 的明确测试夹具，把 `task.events` 与 `task.browser.get` 延迟 400ms。`task.step.get` 返回后，真实任务标题、基础定义列表、浏览器局部加载状态和时间线局部加载状态立即存在；附属查询完成后原位替换。随后完整夹具继续通过审批预览/批准/撤销、文件授权、浏览器交接、时间线分页与局部失败、产物空态/版本变化/分页失败恢复、旧响应丢弃及筛选分页。

结构化结果包含 `progressiveDetail=true`，其余 35 个既有布尔回归项均为 `true`。测试 TaskSpace 1 已结束且未保留页面；夹具不写产品任务库、不含真实正文或完整 Agent Payload。

## 命令与结果

- Ego Lite 隔离页面加载正式 `apps/desktop/ui/index.html` 和 `task-space.js`，调用 `checkTaskSpace(page)`：PASS。
- `/Users/archersado/.cargo/bin/cargo test --offline --locked -p yonder-desktop --lib`：39/39 PASS。
- `python3 scripts/check_architecture.py`：PASS。
- `openspec validate ds-s2-task-overview --strict`：PASS。
- `node --check apps/desktop/ui/task-space.js`、`node --check scripts/verification/desktop/check-task-space.mjs`、`git diff --check`：PASS。

## 边界

本轮没有协议、SQLite、状态所有者或依赖方向变化。真实 Tauri 窗口的主观手感仍可在最新构建中复核，但不影响确定性证明“附属查询未完成时核心详情已经渲染”。Windows 原生验证继续按用户既有安排暂缓，完整 DS-S2 保持 `verifying`。

交付物为空经审阅确认不是本 UI 回归：现有 `task.create` 没有任务输入/预期输出契约，`publish_artifact_manifest` 也只供可信能力用例且未开放 Agent/Gateway 发布。该问题必须先建立跨 AG/TM/能力 Adapter 的 Architecture Decision 与独立 Story，不能由详情页从 Agent 文本推断产物。
