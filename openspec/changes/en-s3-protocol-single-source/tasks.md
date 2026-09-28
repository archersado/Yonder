# 任务

- [x] PROTOCOL_VERSION 下沉至 yonder-protocol；application 再导出；release_contract 路径保持
- [x] CLI hello 握手改引常量，删除 minor 字面量；Codex 桥钉定 1.19 保留并注明
- [x] 新建 application::unknown_reason 模块：内部枚举、唯一 From 映射、逐值序列化回归
- [x] gateway/query 手写映射删除，改用 reason.into()；清理未用 import
- [x] valid_id 单一实现回归 protocol；application 映射 Error 并 pub use
- [x] adapters task_store 的存储字符串转换注明同源于 protocol 派生形状
- [x] workspace 测试与协议生成 --check 全绿（161 项）
- [ ] 独立 Verification Goal：复核等价性断言（wire/schema/依赖方向）与 bridge 例外声明
- [ ] 归档 Change；Story EN-S3 保持 design-review 直至阶段 2 设计审阅通过
