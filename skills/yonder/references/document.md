# 文档任务

Document 只处理 Yonder 已授权的 DOCX、XLSX、PPTX 引用，不接触路径或 OOXML。

## 执行顺序

1. 使用 `task_file_grants` 读取同一任务的本机授权引用。源文件需要 `read`，默认另存目标需要 `create-new`。
2. 使用 `task_file_execute(read)` 取得受限文件事实与当前 SHA-256；不要自行打开路径或解压 OOXML。
3. 调用 `task_document_execute`，传入同任务的源/目标授权、最新 `expected_hash` 和唯一文本替换。默认另存，不把覆盖标志或路径塞入请求。
4. 根据 Yonder 返回的结构校验、锁和提交结果决定完成、失败或等待用户。结果 `unknown` 时不得重复写入。

## 围栏

- 不使用 shell、Python、Office COM、AppleScript、WPS 私有插件或直接 ZIP 修改补齐能力。
- 源哈希变化、目标已存在、Office/WPS 锁、授权过期或文本不唯一时失败关闭，并将最小原因交给用户。
- 覆盖需要单独的本机明确授权；没有正式能力时报告不可用，不改为直接写文件。
- 回执只展示格式、哈希摘要和产物引用，不输出文档正文或内部 XML。
