# AD-DO-01 OOXML语义转换边界

- 状态：Accepted
- 日期：2026-09-17
- Story：DO-S2
- OpenSpec：`do-s2-ooxml-semantic-transform`

## 决定

Application定义不暴露ZIP entry或XML路径的Document Port；Rust Adapter接收OOXML字节并返回有界语义文本或转换后的OOXML字节。首批只支持DOCX、XLSX、PPTX中唯一文本节点的精确替换，目标不存在或出现多次均拒绝。

文件路径、`expected_hash`、文件身份、宿主占用、写租约、临时文件和原子提交仍由FI-S1负责。FI-S1门禁完成前，Document Port不得接文件写入或Gateway；当前实现只能进行无副作用的内存转换和夹具验证。

## 理由

这复用Accepted AD-E0-04的Rust进程内技术栈，又避免Document Adapter复制File Adapter的安全职责。唯一匹配比隐式批量替换更适合作为首个可验证编辑原语。

## 后果

跨多个OOXML文本run的短语、按单元格地址写值、批量编辑、宏和复杂图表暂不支持；后续有真实需求时扩展语义操作，不建立完整OOXML对象模型。
