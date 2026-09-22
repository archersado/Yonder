当前归属 Story：DO-S1；规划：`docs/specs/epic-DO/story-DO-S1/README.md`。旧编号保留历史追溯。

# Verification Goal：E0-S4

复核统一样本、两候选可运行证据、原文件不变、hash 冲突 fail closed、未修改 part 保真、输出结构有效、WPS 实际打开及唯一选型。缺失 DOCX/XLSX/PPTX 任一格式、只验证自行生成样本、或长期保留双实现时，Goal 失败。

当前状态：通过。唯一选择 Rust 进程内实现；Node Worker 淘汰。合成样本与 Microsoft Transitional 复杂样本均通过自动校验，Rust 三种输出均由 WPS 成功打开并经用户目视确认。

2026-09-20 独立复核：自动化结论未漂移，但 WPS 打开结论缺少可审计的原始证据，本 Goal 当前未通过，不得 Archive。详见 `verification-audit-20260920.md`。
