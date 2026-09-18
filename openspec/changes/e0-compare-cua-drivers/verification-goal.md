当前归属 Story：CU-S1；规划：`docs/specs/epic-CU/story-CU-S1/README.md`。旧编号保留历史追溯。

# Verification Goal：E0-S2

复核固定版本、Windows 原始证据、统一 Harness、故障语义、WPS/普通权限文本应用/文件资源管理器用例和唯一选型。测试中混入 Agent、安装未签名构件、整体提权、或结论保留长期双栈时，Goal 失败。macOS 与 Microsoft Office 已按需求变更移入后续 Epic，不阻塞本 Goal。

当前状态：通过。Windows 首版唯一选择 trycua 0.25.0；Qwen 0.20.5 淘汰。已验证只读能力、生命周期错误、普通权限 AX 输入、文件资源管理器动作后观察及异常退出恢复；WPS High integrity 限制已明确 fail closed。

2026-09-18冻结复核：选型结果已同步到正式桌面依赖清单、产品Adapter/Worker和macOS预览资源；证据转由`cu-s2-supervised-dispatch-observe`继续维护。本Goal保持历史PASS，不再追加产品验收。
