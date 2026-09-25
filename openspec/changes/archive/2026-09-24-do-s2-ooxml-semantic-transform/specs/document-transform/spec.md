# OOXML语义转换 Delta

## ADDED Requirements

### Requirement: 有界语义读取

转换器 MUST 返回有界语义文本和文件摘要，不暴露原始OOXML细节。

#### Scenario: 读取有界文本节点

- **WHEN** 调用方提交有效DOCX、XLSX或PPTX字节及文本条数上限
- **THEN** 返回格式、SHA-256、有界文本节点与截断标志
- **AND** 不返回XML路径、命名空间或原始XML

### Requirement: 唯一文本替换

转换器 MUST 只在目标文本节点唯一匹配时生成同格式OOXML字节。

#### Scenario: 唯一匹配替换

- **WHEN** 原文本在该格式正文文本节点中恰好出现一次
- **THEN** 返回可重新打开且包含新文本的同格式OOXML字节
- **AND** 未修改part的解压后字节保持一致

### Requirement: 稳定拒绝

转换器 MUST 对不支持或无法唯一定位的请求返回可区分错误，且不产生文件副作用。

#### Scenario: 目标不唯一

- **WHEN** 格式不支持、输入损坏、参数无效、目标不存在或目标不唯一
- **THEN** 返回可区分错误且不产生文件副作用

### Requirement: 文件边界

在FI-S1通过前，转换器 MUST 保持字节输入/输出边界，不接入宿主文件写入。

#### Scenario: FI-S1未通过

- **WHEN** FI-S1尚未通过
- **THEN** 转换器不得接路径写入、宿主锁、原子替换或Gateway
