# AD-E0-03 ego-lite BUA（Deferred）

- 状态：Deferred
- Story：E0-S3
- OpenSpec：`e0-defer-ego-lite-bua`
- 日期：2026-09-10

## 决定

Windows 首版暂不交付 BUA。Yonder 保留 BUA Bridge 端口，但不实现 ego-lite Runtime、不复制 Browser Task Space，也不引入替代浏览器自动化引擎。

## 原因

ego-lite 官方当前仅支持 macOS，Windows 尚在 roadmap；本项目已将 macOS 验证移出当前 E0 范围，因此不存在可验证、可发布的目标平台 Runtime。

## 重启条件

满足任一条件后重新建立 Story 与 OpenSpec：ego-lite 提供 Windows Runtime；或 macOS 重新进入交付范围。届时必须验证 CLI 部署、Task Space create/reuse/switch/complete、handOff/takeOver、事件读取和 Yonder 状态映射。
