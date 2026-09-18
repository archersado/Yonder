# OOXML语义转换 Delta

## ADDED Requirements

### Requirement: 有界语义读取
- **WHEN** 调用方提交有效DOCX、XLSX或PPTX字节及文本条数上限
- **THEN** 返回格式、SHA-256、有界文本节点与截断标志
- **AND** 不返回XML路径、命名空间或原始XML

### Requirement: 唯一文本替换
- **WHEN** 原文本在该格式正文文本节点中恰好出现一次
- **THEN** 返回可重新打开且包含新文本的同格式OOXML字节
- **AND** 未修改part的解压后字节保持一致

### Requirement: 稳定拒绝
- **WHEN** 格式不支持、输入损坏、参数无效、目标不存在或目标不唯一
- **THEN** 返回可区分错误且不产生文件副作用

### Requirement: 文件边界
- **WHEN** FI-S1尚未通过
- **THEN** 转换器不得接路径写入、宿主锁、原子替换或Gateway
