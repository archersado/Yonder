# 文件身份 Spike Delta

## ADDED Requirements

### Requirement: 别名归并
- **WHEN** 原路径、软链接和硬链接指向同一既有文件
- **THEN** 系统必须识别为同一文件身份，不得授予并发写租约

### Requirement: 受控新输出
- **WHEN** 输出尚不存在
- **THEN** 系统规范化其既有父目录并阻止越出授权根
- **AND** 临时文件与目标位于同一目录，校验后原子提交

### Requirement: 产品门禁
- **WHEN** 双平台身份和Office/WPS锁证据未齐
- **THEN** FI-S1与DO-S2不得开放产品写入口
