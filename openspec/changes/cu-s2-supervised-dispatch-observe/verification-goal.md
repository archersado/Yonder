# CU-S2 受监管派发与强制 Observe 独立 Verification Goal

日期：2026-09-15。关联 CU-S2 DISPATCH-01、OBSERVE-01、UNKNOWN-01、Accepted AD-CU-01/04、AD-TM-08 与 `openspec/changes/cu-s2-supervised-dispatch-observe/`。验证者在实现后运行锁定离线回归和隔离 macOS 原生样本；未修改正式任务库，Windows 按用户要求暂缓。

## 结果

PASS（首批 macOS 后台 AX 输入子范围）。工作区锁定离线回归 36 项通过；新增最小合约检查覆盖完整身份一致、旧 Worker 身份拒绝、后置 Observe 缺失为 unknown 及非法响应拒绝。依赖方向与 `git diff --check` 通过。

[结构化原生证据](../../../apps/desktop/evidence/cua-dispatch-20260915/result.json)表明：Application 从临时 SQLite 的真实已声明步骤原子准备 attempt，随后经产品 `dispatch_prepared` 读取该事实并调用 Rust CU Port；Yonder 监管的无界面 Node Worker 加载固定 `@trycua/cua-driver@0.25.0`，隔离后台输入后 SDK Observe 与原生控件一致，Worker 正常退出。没有启动上游 App、截图或 Recording，证据未保存输入正文或窗口树。

首次最终链路验证因测试工具把待准备 attempt 的 `accepted_sequence` 错写为当前序号，被 Application 按设计在派发前拒绝；[失败证据](../../../apps/desktop/evidence/cua-dispatch-20260915/prepared-sequence-failure.json)保留。修正工具为初始 0 后复测通过，未放宽产品校验。

2026-09-18补充：[正式资源证据](../../../apps/desktop/evidence/cua-formal-runtime-20260918/result.json)确认macOS预览包携带Node、与产品源码一致的Worker及锁定trycua 0.25.0，且不含Qwen依赖；打包和原生验收脚本均从`apps/desktop`/`crates/adapters`正式目录取运行依赖，不再读取Spike依赖树。

[正式宿主责任链证据](../../../apps/desktop/evidence/cua-formal-host-20260918/result.json)确认重新打包并启动的`com.yonder.desktop`经真实UDS协商协议1.12，CUA能力由宿主权限检查报告为available；正式资源校验与既有同一路径连续动作/Observe成功证据共同闭合宿主到Worker责任链。本轮两次动作复测均检测到真实用户输入，按设计返回`unknown/user-input`且没有自动重试；该结果保留为用户输入优先证据，不冒充动作成功。

## 完成边界

本 Goal 证明 prepared attempt 到真实 SDK 动作及强制后置 Observe 的 macOS子链路、固定版本SDK正式预览资源与Yonda宿主权限责任链。正式发布签名与 Windows 证据未完成；CU-S2 保持 verifying，不 Archive。
