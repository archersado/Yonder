# AD-AG-06 云端Connector传输路线

状态：Proposed  
日期：2026-09-18  
关联：AG-S1、AG-S5、AD-VI-02

## 背景

产品简报要求Yonder连接第一方和第三方云端Agent，Architecture Spine限定为Yonder主动建立的单一长期WSS。现有Node样本只证明协议方向，使用临时自签证书并关闭校验，不能成为产品路线。仓库尚无Rust WebSocket依赖，外部平台的配对、端点和设备凭据契约也未给定。

## 候选与边界

首选候选为Rust进程内异步WebSocket客户端，使用系统信任根校验TLS并复用Desktop现有Tokio运行边界；不引入Node Worker、第二App、本地HTTP/TCP监听或第二份协议类型。候选库必须支持WSS、ping/pong、关闭帧、帧大小限制和取消。

Spike只使用固定非敏感标记连接公开测试WSS，退出即清理，不使用测试或产品凭据。产品端点、登录配对、令牌轮换、撤权和系统Credential Store不在Spike内；这些契约缺失时产品Connector保持disabled。

## 统一样本与淘汰门槛

macOS与Windows使用相同样本验证：有效TLS连接、无效证书拒绝、hello后双向协议帧、64 KiB上限、ping/pong、服务端断开、停止后无残留任务、带上限和抖动的指数退避。证据不记录凭据、正文或完整Payload。

任一候选若需要关闭TLS校验、常驻第二进程、开放监听端口、无法在停止时释放、不能限制帧大小或将业务协议复制到Adapter内则淘汰。单平台通过不接受ADR，Windows按用户决定暂缓时本AD保持Proposed。

## 2026-09-18 macOS 子范围结果

`tokio-tungstenite 0.30.0 + rustls ring + macOS系统信任根`样本通过可信WSS回显、Ping/Pong、主动关闭后重连、64 KiB帧配置、无效TLS拒绝和有界退避断言；未启动第二进程或持久化凭据。Windows统一样本仍暂缓，因此本AD保持Proposed。

## 待决

Rust客户端依赖选择、外部平台配对协议、端点来源、设备凭据存储和Outbox恢复握手仍待验证。AG-S5不得在此前自行建立WSS。
