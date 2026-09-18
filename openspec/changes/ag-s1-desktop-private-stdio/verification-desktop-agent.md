# AG-S1 正式桌面本地Agent独立 Verification Goal

2026-09-14；macOS研发Debug。关联AG-S1 AC1/2/4/5及AG-S2 AC2/3/5/6、AD-AG-03/02。实现结束后独立验证，当前状态verifying，生产认证/Windows不在本轮通过范围。

目标：确认共享stdio真实登记与有界连接，再用正式预览小龙私有父进程Agent连接，在系统任务库登记两任务，通过原生AX/CGWindow/鼠标事件验证悬停显示、面板内刷新、移出隐藏。普通启动不开放stdin，release不开放研发flag；完整Story不得Archive。

首次原生验证失败exit7：Agent实际登记及悬停打开均成功，脚本未从AXValue检查静态任务文本，返回实施取证阶段修正脚本。保留apps/desktop/evidence/live-local-agent-20260914/首次证据；不将首次结果改写为PASS。

## 最终结果

macOS研发接线PASS，完整Story仍implementing，不Archive。Desktop3项回归通过，正式二进制和共用stdio测试宿主离线锁定构建通过；架构/文档关联和git diff --check通过。共享stdio自检包含未握手、实际两任务、重试/冲突、四表原子计数，并拒绝无换行与65537字节帧：[共享会话证据](../../../apps/desktop/evidence/local-agent-shared-stdio-20260914/frames-result.json)。

正式桌面Agent连接PID10353，[真实登记证据](../../../apps/desktop/evidence/live-local-agent-retry-20260914/agent-result.json)证明两ID不同、重试不新增且均created。同一进程原生[菜单结果](../../../apps/desktop/evidence/live-local-agent-menu-text-20260914/result.json)全部通过：[原生截图](../../../apps/desktop/evidence/live-local-agent-menu-text-20260914/native-agent-tasks.png)可见两个任务卡片、归属local-test-agent和已创建状态。悬停打开、移入保持、原生AX刷新可操作、移出隐藏、小龙与菜单当前桌面同时可见。

第二次原生检查也失败exit7：AXValue仍以完整按钮聚合文本提供，精确相等无法匹配单独Agent文本。将取证匹配改为包含后通过；生产UI/Gateway没有为通过断言改写状态。首次脚本退出后预览进程也退出，单独重试exit2无目标；改为测试父进程启动独立会话后保留PID10353，随后原生取证通过。所有先前登记证据保留，首次未通过不抹除。

连接EOF后只退出读帧线程，当前小龙仍运行供用户检查。正式库保留两条created测试任务，没有手工插库/删除/改终态。当前尚无Gateway取消/执行能力，不能据created要求播放执行动画。普通启动及release/其他平台不开放stdio研发入口；生产认证、UDS/Named Pipe、外部撤权和双平台原生完成继续保留门禁。
