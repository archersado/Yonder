当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# Verification Goal：OCT-S1 CI 增量

状态：本地 macOS 验证通过，等待实际双平台 CI；不 Archive，不将 OCT-S1 标记 Done。

## 验证范围

- Story：`_bmad-output/implementation-artifacts/OCT-S1-TASK-STATUS.md`，AC 2/3/4 的持续回归入口及仓库研发门禁。
- OpenSpec：`openspec/changes/oct-s1-task-status/specs/task-status/spec.md` 的持续集成增量。
- 架构依据：ARCHITECTURE-SPINE、DEVELOPMENT-AND-CHANGE-MODE、AD-OCT-02。Architecture Impact：conforming。
- 验证对象：依赖方向脚本、PR 关联检查、Windows/macOS 工作流及现有协议生成命令接线；不修改产品实现。

## 2026-09-11 本地证据

环境：macOS，`/usr/bin/python3` 3.9.6；未找到 rustc/cargo，`~/.cargo/bin` 不存在。分支：`story/oct-s1-ci-guards`。

1. `python3 -m unittest discover -s scripts -p 'test_*.py' -v`：8 个测试通过，0 失败。覆盖允许方向、越层依赖别名、开发/构建/Windows 条件依赖、Domain 外部库、未登记模块及本地依赖、缺失/空/重复 PR 字段、失效关联、Story ID 前缀误匹配与路径越界。
2. 读取真实 PR 模板并调用 `check_pr`：当前 OCT-S1 的 Story、Change 和验证记录关联通过。
3. Ruby 标准库 YAML 解析通过，矩阵读取结果为 `macos-14`、`windows-2022`；这不是 GitHub 服务端工作流校验。
4. `git diff --check`：通过。

## 2026-09-11 续作：真实 macOS 验证

经用户授权，按 [Rust 官方安装流程](https://rust-lang.org/tools/install/)安装最小工具链至用户目录，不修改 Shell 配置。环境为 macOS 26.5.1（25F80）、aarch64-apple-darwin；rustc 1.98.1（48a229cea）、cargo 1.98.1（797e8a9bc）。以下结果取代初批“本机无 Rust”的验证限制，保留初批记录用于追溯。

- 先增加失败回归：Windows CRLF 正文被误拒绝；验证记录 OCT-S10 被误认作 OCT-S1；Story 引用相似 Change 后缀时被误判关联。修复前 2 个失败、1 个错误；规范换行并精确匹配标识边界后，11 个 Python 测试全部通过。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py`：真实 Cargo metadata 依赖检查通过。实际 PR 模板关联检查通过。
- `cargo test --workspace --locked`：6 个测试通过，0 失败；构建 37.65 秒，SQLCipher Adapter 两项测试耗时 0.29 秒。覆盖 Domain 生命周期、Application 提交失败、协议非法输入与精度、生成一致性、SQLCipher 并发旧序号/Outbox 回滚、有界恢复。
- `cargo run -p yonder-protocol --example generate --offline --locked -- --check`：通过；未改写生成物。
- 初次沙箱下载因无法连接代理失败；授权联网后下载 Cargo.lock 锁定依赖并完成测试。Cargo.lock 未改变。
- ts-rs 对 `deny_unknown_fields` 的既有忽略警告仍存在；未屏蔽警告，非法输入契约测试通过。

这批结果属于本地 macOS 库层验证，不是 GitHub macos-14 runner 或原生桌面 E2E。

## 尚缺证据

- Windows/macOS GitHub 工作流尚待实际运行；Windows 本批 Workspace 测试和生成检查未执行。历史 Windows 测试不能冒充本批结果。
- 未推送、未创建 PR，未修改远端分支保护。PR 检查读取合成/本地事件，尚无真实 PR 运行证据。
- 本增量仅检查文件关联，不判断证据真实性或全部代码与规格的覆盖关系。前端 import、Adapter 内部互调及原生截图/视频/结构化日志门禁未实施。
- 桌面基础栈 AD-E0-01 未定案；宿主、Gateway、密钥接线和 UI/E2E 仍不满足 Story 完成条件。

实际双平台运行失败时返回实施阶段修复；成功后再更新本记录，Story 其余验收条件通过前不得归档。
