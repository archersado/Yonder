# 独立 Verification Goal：Windows ego-lite active Runtime 选择修复

日期：2026-11-01；Story：BU-S1；结论：**PASS（多版本安装下的可信 active Runtime 选择与真实生命周期）**。

## 失败与根因

Windows 当前同时存在 `0.6.1.13` 与 `0.6.2.5`。ego 当前用户 active-version junction 指向已经完成 bootstrap 的 `0.6.1.13`，PowerShell 的 `ego-browser` 也使用该版本；旧 Desktop 组合根却直接选择安装目录中的最大版本 `0.6.2.5`。因此独立 Bridge 通过 active CLI 可完成真实生命周期，而 Desktop `browser.execute(create)` 超时并按安全语义返回 `-32012`。

首次失败任务 `task_07ca7e7728e0cedb72136455107f6055` 已成为 `interrupted`，attempt 为 `unknown`，没有 Browser 引用；对应测试 Agent 已禁用。只读 ego 空间检查为空，未对未知副作用自动重试，也未直接修改数据库。

## 修复

- Windows 组合根优先读取固定当前用户 `%USERPROFILE%/.local/share/ego/active_version_dir/ego-browser.exe`。
- 使用前 canonicalize 真实路径；最终文件必须仍在固定 `C:\Program Files\Citro Labs\ego lite\Application\<数字版本>\ego-browser.exe` 内。越出安装根、非数字版本父目录、文件名不符或解析失败均拒绝。
- active 指针不可用时才按既有规则选择固定安装根内的最高合法版本。Agent、网页、任务参数与 PATH 不能提供或覆盖 Runtime。
- 未改变固定 PowerShell 模板、私有 Worker、BrowserUsePort、协议、SQLite 或 unknown 不重试语义。

## 证据

- `apps/desktop/evidence/ds-s1-windows-20261101/ego-runtime-selection-result.json`：active canonical 目标为 `0.6.1.13`，仍位于固定安装根；同时存在更高但未激活的 `0.6.2.5`，`active_precedes_higher_unactivated_version=true`。
- 修复前后同一 Desktop Gateway 夹具复跑后，真实 Task Space `ego:7` 创建、完成并关闭，任务 `task_4a5ab80604737e1095a21faaf94fabf2` 最终为 `completed`，`reference_finished=true`。
- 独立 Adapter 以 canonical `0.6.1.13` 完成 create、observe、handOff、takeOver、finish，真实空间 `ego:6` 已关闭；说明 active 目标 Runtime 本身可用。
- Windows Desktop 库测试和构建通过；新增规则测试覆盖安装根、数字版本父目录与精确文件名约束。
- 全程未执行 CUA，未安装或升级 ego-lite，也未回退其他浏览器引擎。

## 边界

本报告关闭 BU-S1 Windows 多版本 Runtime 选择回归，不替代 DS-S1 全状态展示门禁。ego-browser 提示存在可用更新；按照技能要求，本轮未经用户明确同意不执行升级。
