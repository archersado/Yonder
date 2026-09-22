# 任务

- [x] Story三份设计、AD-AG-02与Proposal审阅
- [x] Rust唯一协议与1.1握手门禁
- [x] Application仅Agent登记与同源Gateway分派
- [x] SQLite2→3原子幂等及创建事务
- [x] 实际文件/冲突/回滚/身份版本合约测试
- [x] 私有stdio本地测试Agent联调
- [x] 独立Verification Goal
- [x] macOS生产UDS与首帧Agent身份下的真实MCP登记联调
- [x] macOS原生桌宠面板端到端联调
- [ ] Windows Named Pipe及原生桌宠端到端后续接线
- [ ] 完整Story/PR通过后Archive

## Agent命名增量（2026-09-14）

关联AG-S2、TM-S1、DS-S2三份设计与Accepted AD-TM-07名称技术定稿。Architecture Impact：architecture-change（协议1.3、SQLite schema5）。只落实名称，不授权接管目标或Recording。

要求：1.3 Agent创建必须提供合法name；旧版不接收name且响应不增加name字段；新版本创建/get/list/cancel返回同一名称。名称1–256 UTF-8字节、非空、无控制字符，原样保存。相同owner/key仅在description和name完全相同才返回既有最新快照，否则-32009，不产生记录。取消后重试不恢复任务。

迁移前唯一SQLite备份，事务新增tasks.name与task_creations.name；旧NULL不伪造名称，保留所有业务记录。未知格式、已有加密旧版本、备份或DDL失败拒绝升级。卡片与详情用名称/历史缺名反馈，ID仍可查看。验证名称边界、旧新版门禁、原子回滚、备份与数据保留、正式嵌入UI和macOS原生显示；Windows暂缓、完整Story不归档。

- [x] Rust1.3名称与版本响应兼容
- [x] Application名称贯穿与SQLite5备份迁移/幂等事务
- [x] 桌面卡片/详情Agent名称与历史缺名反馈
- [x] 名称独立Verification Goal与macOS原生证据
