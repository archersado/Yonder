# 独立 Verification Goal：进行中任务先筛选后分页

日期：2026-09-18  
关联 Story：DS-S2  
关联 Change：`ds-s2-running-pagination`  
结论：macOS 子范围 PASS；Windows 按用户要求暂缓，不 Archive。

## 验证结果

- SQLite/Application 合约：在排序更早的非运行任务之后放置唯一运行任务，以页大小 1 请求仍在首个运行页返回该任务。
- 协议兼容：省略 `running_only` 保持旧行为；Gateway 1.15 对新过滤返回 `-32010`，1.16 协商成功并返回唯一运行任务。
- ego-browser TaskSpace 66：前 20 个任务均改为暂停、取消或中断，第 21 个保持运行；“进行中”第 1 页显示第 21 个任务且为 1 项，“全部”仍显示原始分页。
- 正式 macOS UDS：数据库已有 108 个历史任务，未结束列表首 20 项不含新建运行任务；同一时刻 `running_only=true,limit=20` 返回该运行任务，完成后清理为 `completed`。
- 正式原生菜单：重新构建并启动 PID 53034；任务总览“进行中”显示“运行中分页原生验证 / 执行中 / 第1页·1项”。

## 证据

- `apps/desktop/evidence/running-pagination-20260918/result.json`
- `apps/desktop/evidence/running-pagination-native-20260918/result.json`
- `apps/desktop/evidence/running-pagination-native-20260918/native-running.png`

## 自动检查

- `cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop`：48 项通过。
- `cargo test --offline --locked -p yonder-cli`：1 项通过。
- `node --check`：Task Space 产品脚本与 ego-browser 回归脚本通过。
- `scripts/check_architecture.py` 与 15 项门禁单测通过；`git diff --check` 通过。

已知 `ts-rs` 对 `deny_unknown_fields` 的解析警告为既有生成器警告，不影响测试结论。本机工具链未安装 `rustfmt` 组件，因此未执行 `cargo fmt`；本次改动沿用现有文件格式。Windows 原生证据依用户决定继续暂缓。
