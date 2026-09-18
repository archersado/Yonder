# AD-CU-04 受监管动作派发与强制 Observe

状态：Accepted（CU-S2 首批 macOS 后台 AX 输入子范围）；日期：2026-09-15。关联 CU-S2、TM-S2/TM-S3、AD-CU-01/02、AD-TM-08。Architecture Impact：architecture-change（Application CU Port 与受监管 Worker 私有协议）。

## 来源与决定

产品简报“两条执行路径”和补充材料“执行原则”要求 CUA 每一步后 Observe；架构主干规定 Driver 模型无关、单桌面租约、崩溃或超时为 `unknown` 且不得自动重试。用户已明确只使用 trycua SDK，不附带独立应用。首批只把已验证的 `@trycua/cua-driver@0.25.0` 后台 AX 文本输入接入产品 Adapter，不扩展点击、拖动、像素输入、语义规划、人工接管或 Recording。

Application 定义执行请求与结果 Port；CU Adapter 监管一个按需 Node Worker。Worker 只通过继承的 stdio 接收一次完整执行身份和内部可信目标，先 Observe 并取得唯一可编辑元素，再执行一次输入，随后无条件重新 Observe。只有动作结果已知、后置 Observe 有效且完整 task/step/attempt/Worker/host 身份一致时，才返回已知结果；超时、崩溃、断连、非法响应或 Observe 失败统一返回 `unknown`。任何失败均不自动重发动作。

## 边界与安全

Worker 无窗口、托盘、常驻服务或可重连端点；不开放 HTTP/TCP。Node 可执行文件、Worker 脚本和 SDK 入口只由可信组合根配置，必须是绝对普通文件，不能来自 Agent/UI 请求。目标 PID/窗口仅为 CU 内部临时引用；本子范围的原生样本使用隔离测试窗口，不向 Gateway 增加动作协议，也不把 PID/路径暴露给 Agent。

私有 Worker JSON 是 Adapter 内部传输，Rust Application 类型为语义契约；Node 仅做 SDK 参数翻译并严格回显身份。日志与验证证据不保存输入正文、SDK Payload、截图或完整窗口树。Yonder 任务 SQLite 仍是状态事实源，首批不新增数据库字段或外部协议版本。

## 验收与剩余门禁

- DISPATCH-01：完整 prepared attempt 身份才能调用 Port；响应身份不一致不得接受。
- OBSERVE-01：动作返回后必有后置 Observe；Observe 失败不能报告成功。
- UNKNOWN-01：Worker 超时、崩溃、断连和非法响应为 unknown，且不自动重试。
- SDK-01：只加载固定 0.25.0 SDK，不启动或打包上游 App。

独立 macOS 原生验证须使用隔离窗口核对 SDK 后置 Observe 与原生字段一致，并确认 Worker 退出。Windows 按用户要求暂缓，产品 Node/SDK 打包、Yonder 正式宿主权限责任链、可信 WorkRef 完整身份、多 Space/显示器、步骤结果事务、停止/接管与 Recording 仍保留后续门禁；本决定不宣称 CU-S2 完整 Done。
