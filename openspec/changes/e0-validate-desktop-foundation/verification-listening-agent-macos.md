# 收到请求与隐藏态状态唤醒验证

日期：2026-09-15  
关联 Story：DS-S1  
结论：PASS（macOS；Windows 依用户决定暂缓）

## 验证结果

- Application 与桌面组合根测试通过：只有成功的 Agent `task.create` 产生一次非持久化收到请求信号；拒绝不触发，执行中状态优先并丢弃待播信号，1.6 秒后回到数据库派生状态。
- 正式桌面通过私有 stdio Agent 完成 `gateway.hello → task.create → task.cancel`。原生连续截图显示完整小龙、抬头动作与 `✓`；创建任务最终保留为 `cancelled`，没有删除任务数据。
- ego-browser 九态长周期检查通过，收到请求包含呼吸、眨眼与独立点头动作。状态素材在全部解码后才启动轮询，避免 WebView 尚未就绪时消耗瞬时信号。
- 隐藏态状态变化页面检查通过：`idle → waiting_for_user` 唤醒一次；相同状态轮询不重复唤醒；已知态变为 `unknown` 再唤醒一次。
- macOS 原生完整等待生产配置的三分钟闲置：窗口先收至右侧 `56×112` 双眼形态；保持同一 Agent 连接创建任务后恢复为 `200×201`，截图显示完整收到请求形象与 `✓`。结果为 `natural_dock=true`、`woke_on_state_change=true`。

## 证据

- `cargo test --offline --locked -p yonder-desktop -p yonder-application`：全部通过。
- `apps/desktop/evidence/listening-agent-built-20260915/result.json`
- `apps/desktop/evidence/listening-agent-built-20260915/native/`
- `apps/desktop/evidence/listening-state-change-20260915/browser-result.json`
- `apps/desktop/evidence/listening-state-change-20260915/wake-after-ready.json`
- `apps/desktop/evidence/hidden-state-wake-agent-20260915/result.json`
- `apps/desktop/evidence/hidden-state-wake-agent-20260915/docked.png`
- `apps/desktop/evidence/hidden-state-wake-agent-20260915/awake.png`

此前三轮原生收到请求取证未通过，分别暴露了瞬时信号在首个 UI 读取前过期、截图探针启动过慢、以及打包前未显式更新内嵌 UI 的问题；对应失败证据均保留。最终实现把计时起点放到首次 UI 读取，并在素材解码后启动任务轮询；显式构建、打包后的上述证据通过。

本项不新增 wire/schema 或持久化状态，不伪造完成、失败等尚无生产事件源的状态。Windows 原生证据继续暂缓，不据此关闭 DS-S1 的双平台总门禁。
