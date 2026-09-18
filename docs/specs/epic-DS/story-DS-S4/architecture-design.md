# DS-S4 架构设计

## 边界与依赖

Desktop Adapter 负责原生文件选择、ZIP 解包校验、应用数据目录内暂存和资源切换；WebView 只读取已通过校验的本地资源描述，不解析 ZIP。任务状态仍由 TaskHost/SQLite 产生，桌宠现有事件驱动呈现不变。生成委托由 AG-S4 Skill 使用外部 Hatch Pet 式流程完成，Desktop 不调用图像模型或保存模型密钥。

## 状态与契约

导入状态仅存在于当前桌面会话：`idle`、`validating`、`active` 或 `rejected`。它不映射 Task Status，不能覆盖小龙的 `executing`、`waiting_for_user` 等真实生命周期状态。宿主只向 WebView 传递已验证资源根、manifest 版本和有限帧清单；WebView 不接受原始 ZIP 路径或任意资源 URL。

生成请求在生成前增加 `awaiting-export-consent`，经用户明确确认后才成为 `generating`；Agent/Skill 回传产物后进入既有 `validating`。取消、会话断开或失败转为 `rejected` 且删除暂存，不改变当前资源包。生成状态只表达资产流程，不能作为任务状态或桌宠动作来源。

## 导入事务

1. 用户明确选择本地 ZIP，宿主先建立该次导入的受限暂存目录。
2. 逐项校验 archive 路径、条目类型、总大小、manifest 与所有声明帧；任何失败停止并删除暂存目录。
3. 校验完成后，以同一文件系统内的目录原子替换激活包；替换失败保留旧包。
4. 宿主发布无正文的 `assets_changed` 展示事件；WebView 重新加载已验证 manifest。加载失败回退内置包，并显示稳定错误。

manifest 不是协议或任务持久化模型。激活包标识仅属于桌面本地偏好，不能由 Agent、任务、MCP 或云端请求修改。导入日志只记录结果分类、包版本与资源计数，不能记录 ZIP 原路径、文件正文或图像内容。

AD-DS-04 将 Hatch Pet 的身份锁定、逐状态生成、透明度/循环/连续性质检作为 Agent/Skill 的生产契约。Skill 先接收用户选择的参考图与性格说明，再生成 Yonder 的九个固定状态；Hatch Pet atlas/pet.json 必须由确定性转换器输出 Yonder manifest。文件外发、上传副本与保留策略依赖 FI-S1，当前不实现该路径。

## 失败与验证

拒绝绝对路径、`..`、符号链接、重复规范化路径、压缩目录外条目、未知文件类型、损坏图像、超限和不完整状态集。解压总量、单项大小、像素数与帧数均采用固定上限；暂存失败、原子替换失败或运行时加载失败不更改旧包。资源包中没有代码执行入口、HTML、CSS、JS、网络 URL 或动态导入。

最小覆盖有效包导入、路径穿越、压缩炸弹阈值、损坏帧、缺失状态、替换失败回滚、重启读取和减少动态效果静态帧。macOS 需提供原生文件选择与桌宠切换证据；Windows 按用户决定暂缓。Architecture Impact 为 conforming：不增加任务协议、SQLite schema、Agent Gateway 或依赖方向。
