# 云端 Connector Spike 证据

## macOS

2026-09-18 使用独立 Rust 进程内样本完成可信 WSS 回显、Ping/Pong、主动关闭后重连、64 KiB 帧配置、无效 TLS 拒绝和有界退避断言。

运行命令：

```bash
cargo run -q --manifest-path spikes/cloud-connector/Cargo.toml
```

结构化结果见 [`evidence/macos-20260918/result.json`](evidence/macos-20260918/result.json)。样本只发送固定公开标记，不使用账号、设备凭据或产品 Payload。

Windows 按用户决定暂缓；本证据不接受 AD-AG-06，也不授权产品 Connector 接线。
