# EX-S5 已验证执行经验召回

Story: EX-S5
Epic: EX
Status: verifying
OpenSpec: ex-s5-experience-recall

[OpenSpec Change](../../../../openspec/changes/ex-s5-experience-recall/)

## 目标

当执行片段交回慢脑时，Yonder 先按当前失败动作的稳定特征召回少量、经过成功后置条件验证的执行经验，再通过既有 Agent 输入通道把经验提示与 handback 一起交给归属慢脑。慢脑仍须从 Gateway 读取 `task.get` 与 `task.events` 的新鲜事实并提交新计划；召回结果不是事实、计划或执行授权。

## 交付

- Application 定义有界经验记录、召回查询及校验规则。
- SQLite 保存不含正文、截图、坐标、元素索引和临时句柄的建议性经验投影。
- Desktop 在 handback 后、唤醒慢脑前执行有界召回；失败或无结果不阻塞 replan。
- 协议与 Gateway 不新增第二通道，Driver 和 Jev 不读取经验库。
- 覆盖 Application、SQLite 与 Desktop 组合链路测试；建立独立 Verification Goal。

## 非目标

不做向量库、Embedding、知识图谱、自动技能生成、跨设备同步或旧动作重放；不从 `unknown`、未核验动作回执或敏感副作用中学习成功经验。

## 当前结论

Application、SQLite 独立投影线程、20ms 有界召回、AG-S5 慢脑输入与顶部浮窗状态已经接通，并在 macOS 完成独立自动化验证。

2026-10-13 正式 `Yonda.app` 已完成“安全任务 achieved 后晋升 → 同特征任务 not-achieved → 交回前召回 → 顶部持续展示”的产品样本，并修复慢脑投递状态覆盖“已召回执行经验”的问题（`9e6aecb`）。当前 Codex 会话未注册 AG-S8 Agent 输入能力，产品正确停在“已召回执行经验，等待慢脑连接”；恢复受支持的唯一慢脑输入会话后仍需补一次 accepted 投递，Change 暂不 Archive。Windows 原生证据按现有决定暂缓。
