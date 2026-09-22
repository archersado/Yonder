# AD-CX-02 Agent 会话临时输入附件

状态：Accepted
日期：2026-09-20
关联：CX-S2、AG-S5、AD-CX-01、AD-VI-02

## 决策问题

圈选截图如何交给本地或云端 Agent Runtime，同时保持同一 `AgentSession`、不暴露本机路径、不扩大文本正文上限，也不把截图写入任务库、日志或 Outbox。

## 候选决策

在既有双工 `AgentSession` 上增加会话级临时附件传输。发送方先声明 `attachment_id`、MIME、总字节数和内容哈希，再用不超过现有 64 KiB 传输帧的有序分块上传；完整校验通过后，`agent.input` 只携带该 `attachment_id` 引用。接收端只在当前已认证会话内解析引用，不能把本机文件路径、URL或其他会话的附件当作有效输入。

首批只接受单个 PNG/JPEG/WebP，解码前总量上限 4 MiB；每个会话同时最多一个未完成附件。正文继续使用现有 16 KiB 上限。附件从声明到 `agent.input` 确认共用同一 deadline；accepted、拒绝、超时、断连、哈希失败、乱序或取消都清理双方缓冲。未知结果不自动重传或发送第二次输入。

Application 只编排附件暂存与 `agent.input` 引用，Gateway Adapter 负责按本地 UDS/Named Pipe 或云端 WSS 帧发送。两种传输使用同一 Rust 协议类型、配额与确认语义。Desktop、截图 Adapter 和 WebView 不直接写 Socket；任务 SQLite、事件、Outbox和日志不保存正文、分块、哈希或图片。

## Spike 与淘汰门槛

隔离样本使用固定生成的无敏感附件字节，覆盖 1 字节、跨分块和 4 MiB 边界；验证完整接收、哈希一致、乱序拒绝、超限拒绝、断连/超时清零和第二会话不可引用。跨会话拒绝不能读取或删除原会话附件，原会话结束时再清理。证据只记录字节数、分块数、布尔结果和错误分类。

若必须扩大单帧超过 64 KiB、暴露本机路径、持久化截图、建立第二条连接，或任一失败路径残留附件字节，则候选淘汰。Spike 通过后才能更新 AD-VI-02、Rust 协议和 CX-S2 产品 Change。

## 验证结论

2026-09-20隔离Spike与非实现者六行矩阵复核PASS：1字节、跨分块和4 MiB边界完成消费，最大帧64,258字节；第二个/重复开始、超限、乱序、哈希错误均拒绝并清理所属会话；跨会话引用不读取或删除原附件；accepted、rejected、unknown、deadline与断连全部清零。证据不含附件正文、base64或摘要值，且未修改产品协议、Gateway或Runtime。据此接受候选，产品接线仍须独立Change与Verification Goal。
