# Yonda 正式桌面宿主

正式宿主包含小龙与贴身轻量任务menu：轻点小龙打开，拖动只移动，休眠双眼先唤醒；关闭/Escape收起。TaskHost使用系统应用目录、标准文件锁、未加密SQLite与显式启动恢复。固定本机身份只读查询复用Application协议，不对外开放服务。

验证：`cargo test --offline --locked -p yonder-desktop`。

关联：docs/specs/epic-DS/story-DS-S2/；openspec/changes/ds-s2-task-overview/verification-host-core.md。后续GUI必须验证本地窗口，Agent不能复用LocalUser权限。

构建：`cargo build --offline --locked -p yonder-desktop -p yonder-cli`。首次在`apps/desktop/cua`执行`npm ci --ignore-scripts`安装锁定的trycua 0.25.0，再用`python3 apps/desktop/package-macos-preview.py`生成macOS debug预览；应用包仅用于研发，无正式签名发布。GUI只允许task-space本地窗口查询，验证见verification-menu-macos.md。
