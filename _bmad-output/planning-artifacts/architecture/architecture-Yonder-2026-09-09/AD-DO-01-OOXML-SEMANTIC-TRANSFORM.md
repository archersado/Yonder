# AD-DO-01 OOXML语义转换边界

- 状态：Accepted
- 日期：2026-09-17
- Story：DO-S2
- OpenSpec：`do-s2-ooxml-semantic-transform`

## 决定

Application定义不暴露ZIP entry或XML路径的Document Port；Rust Adapter接收OOXML字节并返回有界语义文本或转换后的OOXML字节。首批只支持DOCX、XLSX、PPTX中唯一文本节点的精确替换，目标不存在或出现多次均拒绝。

文件路径、`expected_hash`、文件身份、宿主占用、写租约、临时文件和原子提交仍由FI-S1负责。Accepted AD-FI-01 的 macOS-only Runtime 通过后，DO-S2 可在 Application 组合 Document Port 与 File Port：默认另存使用源快照保护写入，覆盖原文件只允许可信 LocalUser 用例；Document Adapter 仍只处理内存语义，不复制文件身份或锁。

另存期间 File Port 必须持有源文件身份租约与共享宿主锁，并在暂存校验紧邻提交前复核源身份和 SHA-256；源变化时不得产生输出。覆盖使用同一源路径的独占锁、预期身份与 SHA-256。OOXML 暂存内容由 Document Port 重新 inspect，格式一致后才允许原子提交。Windows 与 Agent Gateway 按各自门禁后补。

## 理由

这复用Accepted AD-E0-04的Rust进程内技术栈，又避免Document Adapter复制File Adapter的安全职责。唯一匹配比隐式批量替换更适合作为首个可验证编辑原语。

## 后果

跨多个OOXML文本run的短语、按单元格地址写值、批量编辑、宏和复杂图表暂不支持；后续有真实需求时扩展语义操作，不建立完整OOXML对象模型。macOS 文件 Runtime 完成不自动开放 Agent Gateway，也不把覆盖确认降级为 Agent 布尔字段。
