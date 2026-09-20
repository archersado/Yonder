# AG-S1 产品需求

## 目标与来源

产品简报「产品定义」「MVP主干链路」「Task Space与权限模型」要求外部Agent接入本机工具、任务可见可控；补充材料ego-lite接入参考要求保留外部浏览器Task Space。ARCHITECTURE-SPINE「Agent Gateway」「任务、状态与恢复」约束本地与云端请求进入同一用例、Rust协议唯一来源。AD-OCT-02/04/05规定查询、归属与握手。

后续用户变更：所有任务统一展示、并行按资源限制、有任务才悬停、真实执行使用既有小龙状态；MVP不加密、Windows验证暂缓。采用AD-ST-01未加密SQLite，不把身份校验延期。

## 范围与验收映射

| AC | 验收 | 来源 |
|---|---|---|
| AC1 | 每连接先hello，未握手查询拒绝且不返回任务 | 架构Gateway、AD-OCT-05 |
| AC2 | 可信身份由组合根绑定，请求agent_id无法改身份 | 架构授权、AD-OCT-04 |
| AC3 | 两个Agent各读所属任务，本机UI仍看全部，越权get等同不存在 | 简报Task Space、用户统一展示 |
| AC4 | 正式TaskHost中已恢复的真实SQLite快照进入同一GatewaySession查询，不另建状态源 | 架构任务事实源 |
| AC5 | 重连重新握手，过期、坏格式、版本不兼容明确反馈 | AD-OCT-02/05 |
| AC6 | 后续本地UDS/Named Pipe认证及限额验证后才对外开放，禁止HTTP/TCP | 架构Gateway |
| AC7 | macOS/Windows各自原生证据，暂缓Windows不等于通过 | 研发完成定义 |

首批实施只做AC1–5的宿主库接线；不提供公开传输、任务创建、执行或把Codex会话自动识别为Yonda任务。AC6–7与完整Agent接入保留。真实任务提交另拆Story，不伪造运行任务让桌宠看起来忙碌。

## 问题与目标

连接已认证Agent，按统一Gateway语义读取或创建其任务；创建仅AG-S2承接，不支持人工创建。

## 范围与非目标

本Story范围以上文为准；不绕过认证或把测试上下文当真实连接。

## 验收条件

按本文AC映射验证；未覆盖传输与平台的子范围不能完成整个Story。

## 正式宿主私有stdio研发接入

依据Accepted AD-AG-03及用户本地Agent测试变更：可信父进程显式启动--local-agent-stdio，固定研发Agent身份，真实TaskHost共享，不另开库。AC1/2/4/5复用Gateway门禁；AC6生产认证仍未满足。Agent仅登记created，已有菜单有任务悬停可见、移入可操作、移出消失；坏帧/EOF终止连接，普通启动不开放入口。Windows暂缓；原生证据与stdio证据分别保存。

## 本地 CLI 与 MCP 增量

来源为产品简报「产品定义」、补充材料「Agent Skill + 本地主机 CLI + 应用内 Runtime」以及架构主干「Agent Gateway」。2026-09-15用户确认使用`yonder` CLI承接，运行中的Yonder不直接占用MCP stdio。

- AC8：普通启动的macOS桌面进程在应用数据目录监听仅当前用户可访问的UDS，不开放HTTP/TCP。
- AC9：安装包内`yonder mcp`以MCP stdio连接该UDS；Codex退出只断开连接，不退出桌面进程或产生第二只小龙。
- AC10：MCP暴露任务创建、查询、取消和步骤声明工具，所有请求进入既有Gateway；任务创建继续遵守Agent专属、幂等及名称约束。
- AC11：每条连接先由桥接器完成协议1.4握手，帧最多64KiB；Yonder未运行、连接中断、协议或业务拒绝均返回结构化错误且不输出正文日志。
- AC12：本机连接首个`gateway.hello`的有效`agent_id`绑定其逻辑任务身份，后续请求必须一致；该ID用于归属隔离，不替代当前用户私有UDS认证边界。Windows Named Pipe实现与原生证据按用户要求暂缓。

## 云端 Connector 增量

来源为产品简报“安装后登录账号”“连接第一方、第三方云端Agent”、补充材料“经过认证的云端到设备通道”，以及Architecture Spine单一出站WSS约束。

- AC13：Yonder只主动建立一条长期WSS，不开放公网监听、端口映射、本地HTTP或P2P。
- AC14：外部平台认证与设备配对结果由Connector绑定到`AuthContext`；URL、hello中的`agent_id`和请求字段都不能充当凭据。
- AC15：WSS必须使用系统信任链校验证书；证书、主机名或认证失败时不得建立`AgentSession`。
- AC16：断线使用有界指数退避；未确认副作用不得重试，任务事件只按`last_sequence`和Outbox协议续传。
- AC17：云端与本地请求进入同一Gateway/Application用例；云端断线不终止本地Agent、CLI、Recording或已在本地执行的任务。
- AC18：只有认证并hello声明对应能力的云端会话才能注册；AG-S5的`user_input`直接复用该会话，不另建连接。
- AC19：外部平台配对契约和安全凭据存储未就绪时产品连接保持关闭；不得匿名连接、硬编码令牌、把凭据写入普通配置或日志。

当前用户已延期系统Credential Store密钥接线。协议、TLS与重连可使用隔离的内存测试凭据做Spike，但不能据此开放产品云端连接或宣称登录/配对完成。
