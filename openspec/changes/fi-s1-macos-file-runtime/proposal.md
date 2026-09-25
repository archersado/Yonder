# FI-S1 macOS 受控文件 Runtime

Story：FI-S1。Architecture Impact：architecture-change。关联 Accepted AD-FI-01、DO-S2、TM-S7。

## 动机

macOS Spike 与 WPS 真实锁样本已证明文件身份、授权根、宿主锁和同目录原子提交路线，但产品代码尚无 File Port/Adapter。用户明确 Windows 验证可延期，因此先实现 macOS-only Runtime，并让其他平台明确 unavailable。

## 范围

- Application 定义平台无关文件身份、有界读取、原子写、校验回调、回收站请求与稳定错误。
- macOS Adapter 以规范授权根和 `st_dev + st_ino` 识别文件，归并软/硬链接。
- 新建和替换使用同目录独占临时文件、Yonder 写租约、宿主锁、预期身份/哈希、校验、同步与原子提交。
- 删除只进入系统回收站且只接受可信 LocalUser 用例。

不新增 Gateway/CLI/MCP、协议版本、SQLite、永久删除或确认 UI；不读取、修改或删除用户真实文件。
