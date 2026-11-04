# 独立 Verification Goal：DS-S1 Windows 自动空闲停靠

日期：2026-10-25
Story：DS-S1
Goal：`01a15eda-f12a-7e32-bd6c-2d3336e17351`
结论：PASS（Windows 自动空闲停靠子范围）

## 缺陷与修复

2026-10-24 的 Windows 样本等待190秒后仍为200×200，而直接调用 `pet_dock` 可缩为112×56。实现审查发现首次 `pet_task_state` 被提前到状态素材 ready 之前：Windows WebView 可能在 Rust `TaskState` 注册完成前执行一次失败查询，使前端永久保留 `unknown`，安全门禁随后持续拒绝自动停靠。

本次恢复既有设计顺序：状态素材进入 `frames-ready` 后才执行首次状态读取；只对该无副作用读取以100ms间隔执行最多20次启动重试。耗尽后仍保持 `unknown`，不重试 `pet_dock` 或任何写操作。新增 pet 窗口专用的有界阶段诊断，最多提交10条白名单阶段、模式、任务状态和毫秒值，不含正文、输入或完整 Payload。

## 独立证据

- ego-browser 页面顺序检查：Task Space `7`；首次展示查询发生在 `frames-ready` 之后，只调用1次并得到 `idle`。
- Windows 原生样本一：隔离数据目录、无输入；180.008秒发起停靠，181.401秒由200×200变为底边112×56。
- Windows 原生样本二：应用初始化稳定后将隔离测试窗口无激活移至左上角、无输入；180.014秒发起停靠，181.210秒由200×200变为左边56×112。
- 一次复测期间窗口由 `(1180,596)` 被真实拖动到 `(2305,853)`，日志出现多次交互重置；该样本按“本窗口交互重置三分钟计时”的产品规则排除，不作为失败或通过证据。
- Windows Desktop 测试二进制：库8项、桌面入口4项，共12项通过，0项失败。
- 架构检查：17项通过，0项失败。
- `openspec validate e0-validate-desktop-foundation --strict`：通过。
- 相关文件 `git diff --check`：通过。
- 结构化证据：`apps/desktop/evidence/ds-s1-windows-20261025/idle-dock-result.json`。

## 判定边界

本证据关闭 `verification-windows-20261024.md` 的“自动空闲触发未通过”缺陷，并证明 Windows 底边与左边停靠尺寸。它不替代顶部/右侧、点击/键盘恢复、拖动、九态完整原生链路、性能和 macOS 剩余边缘证据；DS-S1 总 Verification Goal 仍未完成，不 Archive。本轮未执行或修改 CUA。
