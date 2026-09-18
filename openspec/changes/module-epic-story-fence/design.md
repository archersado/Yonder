# 实施设计

关联 EN-S1、AD-DEV-01。docs/specs 作为唯一规划入口；Story README 的 Story/Epic/Status/OpenSpec 为关联字段，不再扫描旧平铺文件。Python 标准库校验 Epic/Story ID 与目录、必需章节和关联路径；PR 强制设计状态达到 ready 或更后，历史 design-review 的 Proposal 只保留证据。

旧 Story 内容保存在新目录 legacy-record.md，旧路径只跳转；七个现有 Change 标记新归属。跨模块旧 OCT-S1 保留 TM-S1 承接，后续功能按模块创建新 Proposal。月度计划不作为 Epic。CI 不判断语义设计质量或替代原生验证。
