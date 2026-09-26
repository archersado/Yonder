# 设计

协议从 Rust 唯一源新增 `document.execute`。Gateway 先认证、校验归属和 1.29 能力，Application 再经 TM-S7 准备 attempt，解析源 read 与输出 create-new 授权，调用 `document::save_as`。源哈希、目标不存在、租约、宿主锁和暂存 OOXML 验证均由既有 Port 复核。

成功只返回格式、输出 hash、大小和 attempt 摘要；失败将稳定分类映射为 observed failure 或 unknown，绝不回显路径、正文或 XML。输出授权解析后不可重放；任务完成仍由归属 Agent 的既有 Gateway 方法提交。
