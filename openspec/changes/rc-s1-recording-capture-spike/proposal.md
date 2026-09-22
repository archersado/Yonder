# RC-S1 手动 Recording 原生采集 Spike

关联 RC-S1 与 Proposed AD-RC-01。首轮物理用户来源验证已失败并淘汰；本轮改用“显式用户控制租约”作为控制平面边界，验证 macOS 原生 listen-only 采集能否在租约内产生无正文 `controlled_session_input`，并可靠执行隐私排除、报告证据缺口及在停止后完全静默。Architecture Impact：spike-only；不修改产品协议、SQLite、任务状态或回放。
