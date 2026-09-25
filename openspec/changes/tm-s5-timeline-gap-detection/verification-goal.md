# TM-S5 历史序号缺口检测 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-timeline-gap-detection`；依据：Accepted AD-TM-16、TM5-AC08。

## 验证目标

- 已授权历史页只返回从 `after_sequence + 1` 开始的连续事件；首项、中间、尾部缺失明确报错，不能跳过或返回正常空页。
- 权限校验先于事件读取；缺口检测只读，不修复、删除或重新编号数据。
- 正常分页、已到末尾的空页和并发新提交不误报；现有 UI 保留已显示时间线并反馈局部错误。

## 环境与结果

- macOS arm64，独立内存 SQLite 夹具：创建连续三事件后仅在测试连接绕过外键以注入 1/2/3 号缺口；旧版及当前编码查询均返回 `-32016`，越权返回 `-32004`，任务序号、事件数及 Outbox 在查询前后不变。
- Application 单元测试验证任务快照略旧但随后读取到连续新事件时不误报；正常单项分页与末尾空页通过。
- ego-browser TaskSpace 119 的测试夹具通过：缺口错误保留已加载事件、显示“任务历史不完整”并可重试；第一次误用不存在的页面地址返回 404，在同一 TaskSpace 改用实际 `index.html` 后通过。测试不访问或修改真实任务。
- `cargo test --workspace --quiet`：90 项通过；`openspec validate --all`：67 项通过；架构检查器测试 18 项通过。
- `cargo build -p yonder-desktop -p yonder-cli` 通过；macOS 本机预览包已重建并重启，宿主进程正常启动（PID 19438）。未对真实任务注入缺口或执行修复。
- 本增量不改变 UI 代码、Driver、系统权限或数据库格式；当前 macOS 本机应用库只读核对未发现序号缺口。Windows 原生验证按用户决定暂缓。

## 结论与保留项

macOS 本机此增量 PASS，可进入本 Change 的 verifying 阶段。完整 TM-S5 的产物身份/不可变清单、总配额、附件清理与删除同步未完成；FI-S1 的 Windows 证据未通过前，不开始依赖文件身份路线的产物 Story。不 Archive/Done。
