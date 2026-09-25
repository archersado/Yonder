# 设计

- Adapter 按 `accepted_sequence` 关联尝试不可变身份，不使用当前 phase 回填历史。
- Application 在协议 1.25 输出可选 `attempt_started`；1.24 及以下缺省，继续执行授权、连续性和编码预算。
- Task Space 只显示 step/attempt，明确为“已准备”；Worker/host 只保留协议审计，不进入界面。
- 不迁移、不回填、不增加写入者；macOS 使用独立正式宿主验证，Windows 暂缓。
