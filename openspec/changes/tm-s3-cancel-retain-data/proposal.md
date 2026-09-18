# TM-S3 取消保留数据

关联docs/specs/epic-TM/story-TM-S3/三份设计及Accepted AD-TM-06。Architecture Impact：architecture-change（撤销清理协议/实验格式安全兼容）。用户明确卡片删除只取消、数据不删除；撤销task.delete，兼容未使用格式4回3，不清理任务记录，既有取消核心复用。完整停止/接管门禁保留。
