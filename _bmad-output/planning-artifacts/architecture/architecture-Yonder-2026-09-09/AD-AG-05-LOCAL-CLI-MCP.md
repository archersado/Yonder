# AD-AG-05 本地 CLI、MCP 与生产 UDS

状态：Accepted（macOS MVP；Windows暂缓）；日期：2026-09-15。Architecture Impact：architecture-change（新增生产本地传输、CLI交付物与MCP适配）。关联AG-S1、AG-S2、AG-S3、AD-AG-01/02/03/04。

来源：产品简报「产品定义」、补充材料「Agent Skill + 本地主机 CLI + 应用内 Runtime」、ARCHITECTURE-SPINE「Agent Gateway」；用户确认由`yonder` CLI承接Codex，运行中的桌面进程不直接开放MCP stdio。

决定：Yonder桌面进程持有唯一TaskHost并在`app_data_dir/agent.sock`监听`interprocess 2.x` Tokio Local Socket。macOS父目录权限0700、Socket权限0600，不开放HTTP/TCP；每个连接必须先发送`gateway.hello`，并以其中通过协议校验的`agent_id`绑定独立GatewaySession。绑定后该连接的全部请求必须匹配同一ID，不能中途切换或由单个后续请求改写身份。MVP以当前OS用户和私有端点作为本机认证边界；该逻辑ID用于任务归属隔离，不声称是细粒度身份认证。Agent注册、撤权和Credential Store仍是后续能力。

新增`apps/yonder-cli`，交付为同一安装包内的命令行可执行文件而非第二个.app。`yonder mcp`使用stdio实现MCP适配，通过共享IPC客户端连接已运行桌面进程；它不打开SQLite、不持有任务状态、不启动桌宠。调用方必须显式提供稳定的`YONDER_AGENT_ID`；MCP工具只映射既有task.create/list/get/cancel/events/step.declare/step.get，启动连接先完成Gateway握手。第三方依赖MCP/CLI契约，不直接依赖原始Socket协议。

2026-09-20 用户变更：不允许把服务端会话固定为`codex-cli`。本决定按上述连接首帧绑定规则更新；不改变端点权限、协议字段、任务所有权或本机认证边界。

传输使用完整换行JSON帧，解析前限制64KiB；坏帧关闭单连接，断连不重试副作用，桌面退出清理端点，重启可覆盖同属当前用户的陈旧Socket。日志只输出固定诊断，不记录正文或完整Payload。Windows保持同一协议并映射Named Pipe，按用户要求暂缓实现与验证；不得将macOS通过写成双平台完成。

2026-10-09 连接态修订（用户变更，Accepted）：`yonder mcp`在自身stdio生命周期内复用一条已握手的Gateway连接，不再为每个工具调用建立瞬时连接。Desktop分别投影“任务通道已连接”和“输入通道已连接”；前者来自已认证Gateway连接的建立/断开，后者仍只来自声明`user_input`的双向会话。两者不得压缩为一个“Agent已连接”布尔值，也不得因为任务通道存在就伪造反向输入能力。该瞬时状态仅驻留内存并由连接事件驱动，不写SQLite、不轮询任务表。

2026-10-12 MCP生命周期修订（Accepted）：正式Codex长会话证明宿主不会在`yonder mcp`空闲退出后按需重启该进程，旧的10分钟空闲回收会把后续工具调用永久暴露为`Transport closed`。MCP进程现与调用方stdio生命周期一致：stdin保持打开时阻塞等待，不设业务空闲超时；stdin EOF、宿主取消或不可恢复的stdio错误才退出。Gateway断开仍只使当前工具调用结构化失败并清除UDS连接，后续调用重新hello；不创建守护进程、不保持第二TaskHost，也不自动重放任何工具请求或副作用。
