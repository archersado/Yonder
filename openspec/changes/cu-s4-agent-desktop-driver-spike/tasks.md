# 任务

## 已完成设计门禁

- [x] 完成独立研究、CU-S4 三份设计与 Proposed AD-CU-07
- [x] 冻结五工作日期限、统一样本和硬淘汰门槛
- [x] 冻结 `agent-desktop` 唯一源码 revision、bridge contract、sample definitions/catalog 与跨字段 evidence schema

## Harness 与契约实现

- [ ] 首次原生运行前冻结 harness commit/dirty diff、依赖锁、toolchain、features、targets、CRT/链接策略与制品哈希
- [x] 建立不接产品 Gateway/SQLite 的隔离 harness 和受监管 Worker
- [x] 建立禁用能力 allowlist/denylist、线性化派发与结果映射契约测试
- [x] 验证 Windows 固定工具链和原生 Backend 所需 COM/DPI 初始化；只读 fixture snapshot 不计为原生动作样本
- [ ] 实现 `IDENTITY-01` 双窗口 fixture 与跨平台同语义 selector；使原生 Worker 只读 snapshot 独立核对 fixture 代际、目标身份和唯一限定引用
- [ ] 验证 Windows WinEvent 生命周期观察器与隐藏窗口事件屏障，覆盖 HWND 复用、队列溢出、观察器故障、Host/Worker 一致性及会话/输入桌面/完整性权限拒绝；未通过前不得开放原生动作

## 双平台原生证据

- [ ] 完成 Windows 原生样本和结构化证据
- [ ] 完成 macOS 原生样本和结构化证据
- [ ] 验证用户/合成输入区分、冻结派发及在途终止后的资源释放
- [ ] 验证干净 Windows、自建制品、SBOM/许可清单和双平台签名责任链

## 独立验证与 ADR 收口

- [ ] 创建非实现者独立 Verification Goal 并逐条复核 ADS-01～10
- [ ] 更新 AD-CU-07 为 Accepted（双平台共享）或 Rejected 结论
- [ ] 若通过，另建产品 Apply Story/OpenSpec；本 Change 不直接接线
- [ ] Archive
