# 圈选捕获 Spike Delta

## ADDED Requirements

### Requirement: 默认不捕获
- **WHEN** 用户未显式启动统一样本
- **THEN** Spike不请求屏幕权限、不截图且不监听鼠标或键盘

### Requirement: 坐标与像素一致
- **WHEN** Spike捕获自身无敏感检查窗口内的已知区域
- **THEN** 返回像素尺寸与逻辑矩形、显示器缩放的换算一致
- **AND** 已知色块校验在容差内通过

### Requirement: 无敏感证据
- **WHEN** Spike完成或失败
- **THEN** 释放捕获像素和临时窗口
- **AND** 证据只含尺寸、坐标、校验布尔值和错误分类

### Requirement: 产品门禁
- **WHEN** Windows/macOS统一样本或ADR尚未通过
- **THEN** 不得新增产品圈选Port、Gateway能力、常驻覆盖层或默认权限请求
