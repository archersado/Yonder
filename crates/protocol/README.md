# 任务只读协议

范围与 wire 约束见 AD-OCT-02。这里只交付进程内契约，不含 IPC、认证或 hello 协商。

## 生成与检查

在仓库根目录运行：

```bash
cargo run -p yonder-protocol --example generate --locked
cargo run -p yonder-protocol --example generate --locked -- --check
cargo test --workspace --locked
```

`generated/` 全部由 Rust 类型生成，请勿手改；测试只比较生成结果，不自动覆盖文件。接入 CI 时执行第二、三条命令。Schema 表达静态结构，Rust 入口执行截止时间、ID 和序号数值范围校验；TypeScript 不替代运行时校验。

ts-rs 12.0.1 对枚举的 serde `deny_unknown_fields` 会输出忽略警告；Rust/Schema 仍保留该约束，契约测试验证未知字段确实拒绝。未为了消除警告关闭输入保护，也未全局屏蔽 serde 警告。

当前 query handler 从 TaskStore 读取状态/序号和事件，未交付步骤、观察、下一步意图。此模块的 agent_id 只是声明；不得在未绑定认证身份的情况下直接暴露给外部请求。
