# 独立 Verification Goal：EN-S1 模块规划与设计门禁

关联 EN-S1、AD-DEV-01、development-fence 场景。环境：macOS 本机 Python 标准库与现有 Cargo 工具链。状态：本地结构与门禁验证通过，待审阅，不 Archive。

## 验证目标

检查 12 个模块 Epic、19 个 Story、每 Story 四份必需文档；七份旧 Story 迁移且保留历史正文；暂停菜单/密钥不被恢复。验证门禁拒绝缺失或空设计、错配模块、未就绪 Story、旧目录绕过、不完整 OpenSpec 和错误双向关联。

## 证据与限制

- `python3 -m unittest discover -s scripts -p 'test_*.py' -v`：15 项通过，0 失败，0.043 秒。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py`：真实 Workspace 依赖和全量规划检查通过。
- 使用真实 PR 模板与仓库路径调用 check_pr：EN-S1 / module-epic-story-fence / 本验证文件关联通过。
- 12 个 Epic、19 个 Story，四份文档齐全；7 个历史正文位于对应 legacy-record.md，旧文件改为跳转。新规划不再引用月度 Epic 为实施入口。

此变更只调整文档及 Python 研发门禁，未修改运行时或 UI，不需要新桌面截图。CI 工作流继续在 Windows/macOS 调用同一脚本，本轮不宣称远端双平台 CI 已运行。结构检查不能证明设计完整性或自动通过历史技术门禁；新目录中的 draft/design-review 仍待逐 Story 设计审阅，不能直接开始功能实施。
