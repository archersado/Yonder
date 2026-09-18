# AD-AG-01 任务仅由Agent经Gateway创建

状态：Accepted（产品入口/权限边界）；任务创建字段级协议、认证Adapter、幂等持久化仍待独立设计审阅，不授权直接实施。关联AG-S2、TM-S1/TM-S2、DS-S2。Architecture Impact：architecture-change（创建权限边界明确）。

来源：2026-09-14用户明确“任务创建是通过agent连接gateway创建，不支持人工手动创建”。覆盖此前材料中本机用户可创建任务的含混建议，不删除用户确认、暂停/取消/接管已有任务的需求。

决定：只有已连接、已认证并握手的Agent可通过统一Gateway创建任务。身份来自可信连接上下文，不能相信请求agent_id。桌宠、任务面板、设置、CLI的人类交互不提供手动新建按钮或绕过Gateway写库。CLI/MCP可作为Agent传输客户端，不转成独立人工创建入口。本地与云端进入同一Application创建用例。任务库、事件、Outbox仍同事务；创建created不代表running，真实准入/派发后才切executing。

创建支持幂等键，归属固定Agent；重复相同创建不得产生第二任务/事件，键相同但意图不同明确冲突。具体字段、任务ID来源、意图范围、幂等保留/迁移与撤权策略须在AG-S2/TM设计定稿后生成OpenSpec。MVP无加密不等于无认证；不开放未认证创建或演示任务入口。
