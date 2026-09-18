# AD-AG-03 正式桌面私有stdio研发接入

状态：Accepted（macOS研发联调）；日期：2026-09-14。Architecture Impact：architecture-change（正式组合根接入生命周期）。关联AG-S1/AG-S2、AD-AG-02、AD-DS-01；不定案生产Socket认证。

来源：产品简报产品定义/MVP主干链路及架构Gateway统一用例；用户要求先以本地Agent测试、继续接通正式小龙。允许通过可信测试Agent父进程启动唯一正式桌面二进制，显式传--local-agent-stdio；入口仅macOS Debug构建启用，release及其他平台不启用。该模式绑定local-test-agent，不从JSON选择身份；父进程私有继承stdin/stdout是研发身份边界，不作为产品凭据认证或CLI人工创建功能。普通启动不读取stdin，不开放端口/Socket，不自动识别Codex任务。

正式进程仍用系统app_data_dir，SQLite恢复、Admission和TaskHost均唯一。stdio工作线程只持现有Arc<Mutex<Option<TaskHost>>>，通过GatewaySession查询/登记；不另开库或建立任务状态。每帧解析前限制64KiB，完整换行，响应沿用Rust协议；诊断仅固定错误，不输出正文。EOF结束连接，重连需重新hello；EOF不改变任务、不重试副作用。退出由系统终止阻塞读取线程；不在GUI线程等待读帧线程，锁仅在完整帧后短持。并发任务执行/Driver仍不在本决定范围。

Agent创建仅created，不自报running；UI依据原有真实快照显示任务。测试会在正式库登记两项明确测试任务，证据只保存计数、布尔和原生菜单结果，不手工删库。当前task.cancel尚无Gateway实现，测试任务保留created供用户检查，不自动伪造终态；后续任务取消按独立Story实施。

验证：共用stdio函数的进程测试验证握手、创建、幂等、有界帧；正式小龙单实例启动后由父进程Agent创建两任务，再原生验证悬停菜单、移入保留和移出消失。生产UDS/Named Pipe/撤权/凭据与Windows仍待设计验证，完整Story不Done/Archive。
