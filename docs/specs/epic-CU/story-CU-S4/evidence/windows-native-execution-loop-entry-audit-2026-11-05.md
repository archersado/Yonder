# Windows 真实执行闭环准入核对（2026-11-05）

## 目标

核对 `IDENTITY-01` 在隔离 Windows fixture 上运行真实 `snapshot → dispatch → observe` 前的门禁。本记录不是动作执行结果；没有向桌面派发输入。

## 当前状态

| 门禁 | 现状 | 判定 |
|---|---|---|
| 固定样本的唯一窗口与元素选择 | `sample-catalog.md` 仅定义“同名双 fixture 窗口”；尚无正式 fixture 制品、窗口创建顺序/身份来源与元素 selector 冻结。`YonderProbeButton` 仅是既有只读诊断 selector | 未满足 |
| 原生 Worker snapshot | `NativeBackend::snapshot` 明确返回 `rejected/unsupported`；Windows 合约测试已验证结构化拒绝及后续零派发 | 未满足 |
| 窗口代际 | Host 当前只读 oracle 证明 PID、HWND、进程启动与当前桌面范围；`window_generation_proven = false`。70,000 次自建隐藏窗口重建未观察到完整 HWND 值复用，但该模式未启动生命周期观察器，不能证明动作路径安全 | 未满足 |
| 原生 dispatch 与 Observe | `NativeBackend::dispatch` 固定返回 `not_delivered`；`observe` 返回 `unsupported`；尚无绑定同一冻结身份的真实动作与独立后置 oracle | 未满足 |
| 用户抢占与取消 | mock Supervisor 合约已有保守映射，真实用户/合成输入分类及原生 Worker 在途清理未验证 | 未满足 |

## 判定

依 AD-CU-07 和 CU-S4 的动作前身份门禁，本次不能把真实桌面动作列为已准入测试。下一步先冻结 `IDENTITY-01` 的双平台 fixture 身份与唯一元素 selector，再使 Worker 原生只读 snapshot 接受且独立校验窗口代际；之后才可打开该样本的真实动作与 Observe。当前结果是 `not_delivered`，不得记为真实执行闭环通过。
