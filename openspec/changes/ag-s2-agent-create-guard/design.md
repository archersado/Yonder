# 设计

先在Application create入口检查AuthContext::Agent，失败PermissionDenied，不访问TaskStore。再执行既有ID/身份校验与事务创建。query错误编码复用已有-32003，Rust仍为唯一响应模型。可信Agent身份只能由认证组合根提供，不能从请求JSON构造。

AG-S2首批产品/架构/调用设计已经审阅；参考AD-AG-01。实际SQLite负例检查LocalUser拒绝且task.get不存在，之后双Agent合法创建和已有Gateway归属回归不变。本机用户对已有任务的读取/控制权限不因创建门禁消失。完整任务创建协议/幂等/认证/接管采集仍不在本变更内。
