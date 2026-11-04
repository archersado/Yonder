# AD-AG-05 本地 CLI、MCP 与生产 UDS

状态：Accepted（macOS UDS / Windows Named Pipe）；日期：2026-09-15，Windows 增量：2026-10-24。Architecture Impact：architecture-change（新增生产本地传输、CLI交付物与MCP适配）。关联AG-S1、AG-S2、AG-S3、AD-AG-01/02/03/04。

来源：产品简报「产品定义」、补充材料「Agent Skill + 本地主机 CLI + 应用内 Runtime」、ARCHITECTURE-SPINE「Agent Gateway」；用户确认由`yonder` CLI承接Codex，运行中的桌面进程不直接开放MCP stdio。

决定：Yonder桌面进程持有唯一TaskHost并在`app_data_dir/agent.sock`监听`interprocess 2.x` Tokio Local Socket。macOS父目录权限0700、Socket权限0600，不开放HTTP/TCP；每个连接必须先发送`gateway.hello`，并以其中通过协议校验的`agent_id`绑定独立GatewaySession。绑定后该连接的全部请求必须匹配同一ID，不能中途切换或由单个后续请求改写身份。MVP以当前OS用户和私有端点作为本机认证边界；该逻辑ID用于任务归属隔离，不声称是细粒度身份认证。Agent注册、撤权和Credential Store仍是后续能力。

新增`apps/yonder-cli`，交付为同一安装包内的命令行可执行文件而非第二个.app。`yonder mcp`使用stdio实现MCP适配，通过共享IPC客户端连接已运行桌面进程；它不打开SQLite、不持有任务状态、不启动桌宠。调用方必须显式提供稳定的`YONDER_AGENT_ID`；MCP工具只映射既有task.create/list/get/cancel/events/step.declare/step.get，启动连接先完成Gateway握手。第三方依赖MCP/CLI契约，不直接依赖原始Socket协议。

2026-09-20 用户变更：不允许把服务端会话固定为`codex-cli`。本决定按上述连接首帧绑定规则更新；不改变端点权限、协议字段、任务所有权或本机认证边界。

传输使用完整换行JSON帧，解析前限制64KiB；坏帧关闭单连接，断连不重试副作用，桌面退出清理端点，重启可覆盖同属当前用户的陈旧Socket。日志只输出固定诊断，不记录正文或完整Payload。

2026-10-24 用户恢复 Windows 验证。Windows 使用 `interprocess` namespaced Local Socket 映射 Named Pipe，沿用同一 hello、连接身份、帧限制、TaskHost 和 MCP Client；不开放 TCP/HTTP，不建立第二个状态宿主。Named Pipe 使用当前用户进程的默认私有 DACL，名称固定由安装包生成，不接受外部覆盖。Windows 原生 E2E 通过前完整 Story 仍不得 Archive。
