# AD-AG-05 本地 CLI、MCP 与生产 UDS

状态：Accepted（macOS MVP；Windows暂缓）；日期：2026-09-15。Architecture Impact：architecture-change（新增生产本地传输、CLI交付物与MCP适配）。关联AG-S1、AG-S2、AG-S3、AD-AG-01/02/03/04。

来源：产品简报「产品定义」、补充材料「Agent Skill + 本地主机 CLI + 应用内 Runtime」、ARCHITECTURE-SPINE「Agent Gateway」；用户确认由`yonder` CLI承接Codex，运行中的桌面进程不直接开放MCP stdio。

决定：Yonder桌面进程持有唯一TaskHost并在`app_data_dir/agent.sock`监听`interprocess 2.x` Tokio Local Socket。macOS父目录权限0700、Socket权限0600，不开放HTTP/TCP；每个连接建立独立GatewaySession并固定绑定`codex-cli`身份，不接受请求选择身份。MVP以当前OS用户和私有端点作为本机认证边界；细粒度Agent注册、撤权和Credential Store仍是后续能力，不能由请求字段代替。

新增`apps/yonder-cli`，交付为同一安装包内的命令行可执行文件而非第二个.app。`yonder mcp`使用stdio实现MCP适配，通过共享IPC客户端连接已运行桌面进程；它不打开SQLite、不持有任务状态、不启动桌宠。MCP工具只映射既有task.create/list/get/cancel/events/step.declare/step.get，启动连接先完成Gateway 1.4握手。第三方依赖MCP/CLI契约，不直接依赖原始Socket协议。

传输使用完整换行JSON帧，解析前限制64KiB；坏帧关闭单连接，断连不重试副作用，桌面退出清理端点，重启可覆盖同属当前用户的陈旧Socket。日志只输出固定诊断，不记录正文或完整Payload。Windows保持同一协议并映射Named Pipe，按用户要求暂缓实现与验证；不得将macOS通过写成双平台完成。
