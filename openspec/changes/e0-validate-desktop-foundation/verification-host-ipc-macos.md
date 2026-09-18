# DS-S1 同进程宿主IPC独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：既有design「Rust Core同进程运行Local Socket echo服务」、spec「CLI执行echo」及AD-E0-01本轮边界澄清。状态：macOS宿主接线、同用户往返、错误拒绝和正常退出清理通过；不代表正式Gateway、跨用户或Windows完成。

## 实施范围

之前桌宠与IPC分别是独立程序，无法证明宿主持有Socket。本轮将IPC协议与传输提取至同一Spike的src/lib.rs，CLI和Tauri复用唯一Rust Message类型。新增可停止、可join的验证线程，Tauri正常Exit先停止服务，释放未完成连接与listener，再等待线程结束。没有引入第二协议模型，没有接入正式任务、Application状态、数据库或密钥。

请求最多64KiB，错误/过大输入关闭本连接，宿主服务保持可用。单次server命令保持原有“一次请求后退出”语义。宿主绑定时不覆盖既有端点，初始化失败即失败启动。此服务串行处理echo，不代表生产并发Gateway；客户端保持半个请求会阻塞后续请求，但不会阻止宿主退出，生产deadline/并发仍属于后续Gateway契约。

Tauri锁文件新增既有IPC库及其传递依赖，没有升级已有锁定包。首次离线解析完成后使用--locked构建，未访问网络。

## 代码与合约验证

- IPC库2项测试通过：Rust消息往返，以及错误版本/超过64KiB拒绝后正常请求仍可用、未完成连接下停止清理端点。初次沙箱绑定UDS报EPERM；同一测试在获准的沙箱外运行通过，不把沙箱失败改写为功能成功。
- 独立CLI脚本三种用例回归通过：正常、错误版本、无效JSON及端点清理。
- 桌宠Release两项原有门禁/最近边缘测试通过；Release离线锁定构建成功。
- 架构与关联检查通过，其作用范围不扩大为隔离Spike全部依赖验证。

## 原生宿主证据

通过系统.app入口启动PID88825。`check-host-ipc.py 88825 …/host-ipc-20260913.json`退出0：lsof确认测试UDS由该宿主PID持有；宿主无TCP套接字；错误版本连接被关闭，随后两次独立CLI echo均成功。

二进制SHA-256：`1cdf0a38eebbc3f9c3c47fa1c0a0441b337d4597af327d6421e2f1a50f4dcb2a`。

随后check-exit-macos.py通过原生AX托盘退出：同资源组29939的宿主88825、GPU88830、Networking88831、WebContent88832全部结束，观测约0.555秒。端点不存在断言通过，无需验证脚本删除产品端点。

证据位于spikes/desktop-foundation/evidence/：host-ipc-20260913.json、host-ipc-exit-20260913.json、host-ipc-endpoint-cleanup-20260913.json。

正常.app重启为PID92356，CLI `client restart-probe` 返回version=0.1及相同固定样本，确认端点可重建。当前运行新版。

## 未关闭项

Windows用户暂缓；跨用户权限仍缺不同用户执行授权。正式任务忙碌接线的循环门禁没有被本次受限echo接线解除。此次增加常驻IPC线程和依赖后，旧版117MiB/123MiB性能读数不能直接作为新二进制预算通过证据，需按已有同组五分钟方法复测。整体Goal仍未通过，不Archive。
