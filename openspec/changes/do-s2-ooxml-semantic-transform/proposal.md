# DO-S2 OOXML语义转换 Proposal

## 问题

Agent尚无不接触XML细节的DOCX/XLSX/PPTX读取与编辑能力；文件安全前置又未完成，不能直接开放路径写入。

## 变更

在Application定义Document Port，在Rust Adapter实现OOXML格式识别、有界文本读取和唯一文本替换。转换仅处理内存字节，不接Gateway或真实文件写入。

## 非目标

不实现路径授权、锁、原子提交、覆盖、协议、任务编排、复杂版式编辑或自动降级CUA。
