# 设计

Application 在 Document Port 与 File Port 之上提供 `inspect_file / save_as / overwrite_local`。文件读取快照是路径、身份、字节和 SHA-256 的唯一输入；Document Adapter 不接触路径。调用方的 `expected_hash` 必须与快照一致后才转换。

`save_as` 使用 File Port guarded write：源身份租约和共享宿主锁保持到输出提交，Document 校验器在暂存阶段重新 inspect 输出并确认格式，同时紧邻提交复核源身份/哈希。目标固定为 `create-new`，存在即拒绝。

`overwrite_local` 在 Application 先拒绝 Agent 身份，再以 File Port `replace` 模式使用源身份与哈希独占锁。两条路径都比较 Document Transform 的输出哈希与 File Receipt，任何不一致返回 unknown，不自动重试。

错误保持 `DocumentError` 与 `FileError` 分类，并额外区分 expected-hash conflict 和 permission denied。正文、OOXML、路径和完整Payload不写日志。
