# DS-S2 宿主核心独立 Verification Goal

日期：2026-09-14；Story：DS-S2；Change：ds-s2-task-overview；AD：AD-DS-01、AD-ST-01。状态：本机宿主核心子范围PASS，完整GUI/产品未通过，不Archive。

实现后独立检查，未修改实现。执行`cargo test --offline --locked -p yonder-desktop -p yonder-application -p yonder-adapters`：desktop 3、application 6、adapters 10项通过，19项、0失败。宿主合约在apps/desktop/src/lib.rs。

两个不同Agent真实SQLite任务由running显式恢复为interrupted、sequence=3，查询两者可见；重复打开不追加恢复。第二宿主同进程及独立子进程均因OS文件锁被拒绝。伪造请求身份返回-32003，无任务信息。错误格式启动失败保留文件字节并释放本次锁，测试清理自己的错误样本后可重新启动。

架构依赖/规划关联、git diff --check通过。Cargo.lock仅登记本地桌面成员及两个已有本地依赖，无新第三方库。Application复用唯一协议编码，desktop不直接依赖protocol。

本次为组合根库，未接GUI或替换现有小龙预览。目录只允许可信调用方提供系统应用数据绝对路径；锁限定同一host.lock，绕过组合根直接开库的工具不受监管。固定文件名符号链接被拒绝，不宣称已实现File Tools全部路径边界。LocalUser固定为desktop且不对外开放，不代表AG认证完成。

Admission容量4为工程选择，未开放执行；activity是观察而非隐藏许可，正式收起仍需预约协调。后续Tauri命令须验证本地task-space窗口，完成轻量入口与系统目录接线。Windows继续暂缓，无本轮GUI证据或PR，不Archive。
