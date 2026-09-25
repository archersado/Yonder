# TM-S5 接管定位历史事实 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-historical-focus`；依据：Accepted AD-TM-19、TM5-AC02/07/08。

## 目标

- 已提交的 `locating` 与最终 `focused/failed` 按原事件序号和同一控制身份形成不可变历史，当前投影不覆盖中间事实。
- 历史、任务状态、当前控制投影、事件和 Outbox 同事务；任一写入失败全部回滚。schema 18→19 不伪造旧任务历史。
- 协议 1.22 隐藏新增字段，1.23 才返回；归属授权、事件连续性、分页及编码预算不变。
- Task Space 明确区分定位中、定位成功和带稳定原因的定位失败，不推断 Recording、用户输入或交回完成。

## 结果

- SQLite/Application/Gateway 回归覆盖成功与失败链、历史写入故障回滚、损坏枚举拒绝、schema 18→19 零回填，以及 1.22/1.23 协议隔离。全仓 92 项 Rust 测试通过；Rust 生成的 TypeScript 与 JSON Schema 为最新版本。
- macOS WebKit 隔离失败夹具显示“正在定位任务工作”与“定位失败 · 缺少辅助功能权限”，证据在 `apps/desktop/evidence/tm-s5-historical-focus-webkit-20260924/`；不连接真实任务库。
- 独立 bundle identifier `com.yonder.focus.fixture` 的正式 Tauri 宿主从专用 SQLite 读取真实提交链。私有 Unix Socket 在 1.22 返回 8 条事件但零条定位事实，1.23 精确返回 `#7 locating`、`#8 focused`，共同绑定 `control_3`。原生 Task Space 显示两条历史并截图；Gateway 与原生 UI 证据分别位于 `apps/desktop/evidence/tm-s5-historical-focus-native-host-20260924/` 和 `apps/desktop/evidence/tm-s5-historical-focus-native-ui-20260924/`。
- 正式用户库只执行结构迁移：schema 18→19、`PRAGMA integrity_check=ok`、历史表为 0 行，证明未从当前投影回填；迁移前快照为 `tasks.db.pre-attempt-v18-72036-1790236595730985000.db`。未写入夹具任务，普通预览宿主已恢复运行。
- 70 项 OpenSpec、架构关联检查、Task Space 前端检查与 `git diff --check` 通过；隔离进程已关闭。

## 结论

macOS 子范围 PASS。Windows 原生证据按用户决定暂缓，因此本 Change 保持 verification-pending，不 Archive；完整 TM-S5 的产物、保留、录制和交回仍未完成。
