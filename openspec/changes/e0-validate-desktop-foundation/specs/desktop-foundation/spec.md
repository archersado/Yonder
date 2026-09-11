# 桌面基础栈增量规格

## ADDED Requirements

### Requirement: 目标平台桌面能力验证

验证程序 SHALL 在 Windows 与 macOS 上证明透明无边框置顶窗口、托盘和基础生命周期可用。

#### Scenario: 启动验证程序

- **WHEN** 验证者在目标平台启动程序
- **THEN** 桌宠窗口与托盘均可见且可交互
- **AND** 退出后不得残留受管进程

#### Scenario: 拖动桌宠窗口

- **WHEN** 用户按住紫色桌宠区域的鼠标左键并移动
- **THEN** 无边框窗口通过 Tauri 原生窗口拖动跟随移动，松开后停止
- **AND** 不选中文字，仅 `pet` 本地窗口获得启动拖动权限，不增加其他窗口权限或依赖
- **AND** 此验证不包含重启后位置恢复；Windows/macOS 分别保留原生验证结果

### Requirement: 本地 IPC 验证

验证程序 SHALL 使用 Local Socket 完成同用户进程间的版本化 JSON 往返，且不得监听 TCP 端口。

#### Scenario: CLI 执行 echo

- **GIVEN** 桌面 Core 正在运行
- **WHEN** CLI 发送带版本的 echo 请求
- **THEN** Core 返回相同 payload 与服务端版本
- **AND** 非当前用户不得获得访问权限

### Requirement: 性能证据

验证程序 SHALL 在每个目标平台记录启动、空闲 CPU、空闲内存和窗口事件延迟。

#### Scenario: 执行空闲基线测量

- **WHEN** 程序稳定空闲五分钟
- **THEN** 证据包含测量方法、环境和原始结果
- **AND** ADR 明确说明是否达到架构预算
