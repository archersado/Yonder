# AD-FI-01 文件身份与原子写入

状态：Accepted（macOS-only Runtime，Windows 与 Agent Gateway 后补）
日期：2026-09-17  
关联：FI-S1、DO-S2

## 决策问题

路径字符串不能作为同文件互斥身份。软链接、硬链接、大小写和重命名可能让不同路径指向同一文件；Document写入还必须识别宿主锁并在同卷原子提交。

## 决策

输入必须是规范化绝对路径；本次接受 macOS 使用 `st_dev + st_ino` 的产品 Runtime。Windows 的 volume serial + file index 在实机统一样本通过前保持 unavailable。新输出先规范化既有父目录并拒绝越出授权根。写租约按文件身份而非路径持有；软链接和硬链接不得取得第二写租约。尚不存在的目标以规范父目录身份和文件名建立临时写身份。

读取和写入单文件上限为 16 MiB。既有文件读取返回内容、文件身份和 SHA-256；替换必须同时携带预期身份与预期 SHA-256，新建必须明确 `create-new`。写入使用目标目录内独占创建的临时文件，完成调用方格式校验、文件 `sync_all` 后原子 rename，并同步父目录。目标存在、身份或内容变化、文件锁、跨卷、权限或提交结果不明均稳定失败，不自动重试。

删除只允许可信本机用户用例携带预期文件身份调用，macOS 使用系统 `NSFileManager.trashItem` 移入回收站；Agent 不能提交确认字段取得删除能力。永久删除不提供产品入口。移动后无法确认原路径消失和回收站结果存在时返回 unknown，不自动重试。

macOS 使用 `File::try_lock` 检测宿主占用。WPS真实DOCX对照已证明打开时拒绝锁、关闭后可取得；无需依赖进程名或旁车锁文件。Application 定义平台无关 File Port，请求不直接信任 Adapter 返回前的路径身份；Adapter 不访问任务库、不写日志。Agent Gateway、任务 attempt 和确认 UI 由后续独立 Change 接线。

## Spike与平台门禁

统一样本覆盖软链接、硬链接、父目录逃逸、同文件锁与同目录原子替换；macOS WPS 真实打开文件锁样本已通过。用户明确 Windows 验证可延期，因此本 ADR 只接受 macOS Runtime 子范围；Windows 仍不得编译或注册未验证路线。FI-S1/DO-S2 的 Agent Gateway 写入口仍受文件授权引用、任务生命周期和可信覆盖/删除确认门禁，不因本决定自动开放。
