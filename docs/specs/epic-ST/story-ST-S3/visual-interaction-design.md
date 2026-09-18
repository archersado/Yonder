# ST-S3 视觉交互设计

## 入口与流程

无UI。可信组合根显式调用open_unencrypted(path)，不接受UI传密钥或回退模式；成功返回TaskStore，沿用Application查询/恢复。

## 状态与错误反馈

合法库可用；打不开返回StorageUnavailable，调用方显示存储不可用，不显示暂无任务、不开放执行。没有迁移进度或新窗口。

## 无障碍与平台验证

不增加原生UI/权限/Driver。验证为真实SQLite Adapter合约及Application恢复集成；macOS本机执行，Windows当前验证按用户要求暂缓，不声称已通过。核心通过后独立记录，不归档完整产品Story。
