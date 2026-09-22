# Cloud Connector Spike Delta

## ADDED Requirements

### Requirement: 出站WSS技术验证

Spike MUST 验证客户端只主动建立WSS连接，并使用系统TLS校验服务端身份。

#### Scenario: 建立出站WSS连接

- **WHEN** 运行云端Connector统一Spike
- **THEN** 客户端只主动建立WSS并使用系统TLS校验
- **AND** 不开放本地监听、不启动第二进程、不保存测试凭据

### Requirement: 连接生命周期

Spike MUST 在停止或断连后释放全部连接资源，并使用有界退避处理重连。

#### Scenario: 服务端断连

- **WHEN** 服务端断开或Spike收到停止信号
- **THEN** 当前连接、心跳与重连计时器都必须释放
- **AND** 重连采用有上限和抖动的退避，不重放未确认副作用

### Requirement: 产品门禁

在外部平台配对、端点和设备凭据契约定案前，产品Cloud Connector MUST 保持disabled。

#### Scenario: 契约定案前保持disabled

- **WHEN** 外部平台配对、端点或设备凭据契约尚未完成
- **THEN** 产品Cloud Connector保持disabled
- **AND** 不得以Spike自签证书、匿名连接或明文凭据替代
