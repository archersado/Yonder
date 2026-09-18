# AD-E0-03 ego-lite BUA

- 状态：Accepted（macOS Bridge；Windows 暂缓）
- Story：E0-S3
- OpenSpec：`e0-defer-ego-lite-bua`
- 日期：2026-09-10

## 原决定

Windows 首版暂不交付 BUA。Yonder 保留 BUA Bridge 端口，但不实现 ego-lite Runtime、不复制 Browser Task Space，也不引入替代浏览器自动化引擎。

## 2026-09-16 重启决定

macOS 已重新进入实际交付与原生验证范围，满足本 AD 的重启条件。BU-S1 在 macOS 直接调用已安装 ego-lite 的 `ego-browser nodejs` 嵌入式 SDK 入口，复用其 Task Space 的 create/reuse、observe、handOff、takeOver 与 finish；Yonder 只实现 Bridge，不嵌入 Runtime、不复制页面/标签/所有权状态机、不用 CUA 回退。CLI 路径只由可信组合根提供；固定 Worker 与经 JSON 双重转义的数据字面量组成 SDK 程序，写入权限0600的短生命周期文件作为标准输入，启动后立即unlink，名称不能逃出字符串，不能拼接为 Shell。

Bridge 每次返回稳定的 `external_task_ref`、所有权和托管页数量；失败收敛为 dependency unavailable、timeout、invalid response 或 runtime failed，不自动重发可能有副作用的调用。Task 执行仍由同一 Application 准入、尝试身份、步骤边界控制和结果提交负责；本增量不另建任务状态源，也不提前实现尚未定案的外部引用持久化。

Windows 仍明确返回能力不可用，不宣称双平台 BUA。待 ego-lite 提供 Windows Runtime 后补该平台的独立 Story 验证，不以 macOS 证据代替。

## 原因

ego-lite 官方当前仅支持 macOS，Windows 尚在 roadmap；本项目已将 macOS 验证移出当前 E0 范围，因此不存在可验证、可发布的目标平台 Runtime。

## 重启条件

满足任一条件后重新建立 Story 与 OpenSpec：ego-lite 提供 Windows Runtime；或 macOS 重新进入交付范围。届时必须验证 CLI 部署、Task Space create/reuse/switch/complete、handOff/takeOver、事件读取和 Yonder 状态映射。
