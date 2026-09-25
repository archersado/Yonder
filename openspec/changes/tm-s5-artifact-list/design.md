# 设计

## 协议与权限

- 协议 1.26 新增 `ArtifactParams` 和 `task.artifacts`，字段为任务、固定清单版本、排他 ordinal 与 1..100 limit。
- `QueryResult::ArtifactManifestPage` 只返回固定版本、条目和可选续页 ordinal；条目可用性使用 Rust 枚举派生 Schema/TypeScript。
- 已认证 Gateway 会话只有在协商 1.26 且存储支持审计时接受方法；Application 继续以当前 `AuthContext` 校验任务读取权限。

## Task Space

- 先读取任务详情的当前清单摘要，再读取该版本第一页；不跨版本拼接。
- 每项显示序号、用户可读可用性和次要 reference ID，不生成路径或打开按钮。
- 分页失败保留已加载项并局部重试；选择轮次保护迟到响应。
- 只有 `current_manifest.version > confirmation.manifest_version` 时显示变化提示。

## 兼容与非目标

- 1.25 及以下不能调用新方法，既有响应不新增偶发字段。
- 不新增 schema、不开放发布 wire、不解析文件、不声称真实能力已经产出清单。
