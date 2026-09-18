# AG-S1 架构设计

## 决策与依赖

依据AD-OCT-05连接内门禁、AD-DS-01宿主研发准入、AD-ST-01MVP任务库。Architecture Impact：conforming，未变更协议、持久化或状态所有者。

TaskHost拥有已恢复SQLite和唯一Admission；GatewaySession拥有连接内AuthContext/协商标记。正式组合根query_session接收可信Rust会话，调用Application的GatewaySession.handle_encoded；编码沿用Rust协议encode，不由desktop手写JSON。desktop仍只依赖Application/Adapters，Application不依赖TaskHost或具体存储。

UI的LocalUser查询入口继续固定本机身份；Agent不得走此入口。会话身份只由未来完成认证的Adapter/组合根建立，agent_id字段不是凭据。本批库方法不是Tauri命令或网络入口，不接受UI提供身份。

## 调用、失败与验证

可信调用方建立会话→hello→task.get/list/events；请求仍带request_id、agent_id、capability、deadline。相同会话每次重新校验；重连创建新会话。编码/存储不可用返回宿主错误，JSON-RPC错误保留协议编码。不记录正文、完整Payload或查询结果。

实际SQLite双Agent样本验证未握手拒绝、各读所属、越权不可见、重连未握手。已有Application合约覆盖坏格式/版本/过期。本批不创建传输/Sidecar，独立库验证不能代替AC6–7。实际认证与传输设计明确后另做Apply，不通过设备密钥暂缓绕过认证。
