# AD-TM-05 用户删除已结束任务

状态：Superseded；由Accepted AD-TM-06覆盖，以下仅保留撤回设计历史，不授权实施。日期：2026-09-14。Architecture Impact：architecture-change（协议/持久化）。来源：产品简报Task Space权限模型、架构任务事实源/保留边界、用户要求任务卡片删除；删除语义澄清未获其他选择，采用显式默认清理任务及历史并保留最小防重放标记。不是仅隐藏列表。

用户确认后仅可信LocalUser可删除终态任务；Agent不得删除，活动状态先取消/停止，宿主Busy/Unknown拒绝。UI使用HTML原生dialog确认说明任务/历史清理、不能恢复及防重放标记保留；不自动确认真实数据删除。接管/Recording前置不因新增按钮解除。

Rust协议新增task.delete/TaskDelete并复用CancelParams相同字段（统一身份/能力/deadline、task_id、expected_sequence），复用kind=deleted/task_id响应。此为本机用户控制，不在Agent hello宣称删除能力，不升级Agent1.2。普通query没有真实宿主活动证据时拒绝；正式TaskHost提供自身ActivityState。唯一Rust模型派生Schema/TS。

SQLite4从3同事务加tasks.deleted（默认0）、events.kind（默认transition）。删除同一IMMEDIATE事务：校验原状态/expected_sequence及终态，删除原Outbox/事件，清空task_creations.description；保留task id/owner/state并置deleted=1、sequence加1，插入无正文deleted事件（执行状态不变）及Outbox。其为删除控制事实，不伪装Domain状态迁移。用户明确手动删除历史时处理旧Outbox，新的删除通知仍保留，未来同步消费者必须识别kind=deleted；不自动清理未同步数据。没有轨迹/附件的首批任务仅清理当前已有任务说明/事件；未来附件关联建立前必须更新清理登记协议，不擅自清理文件。

正常list/get/events/running均不可读deleted标记；commit排除deleted。创建幂等遇deleted标记返回-32013任务已删除，不恢复旧描述、不创建新任务。Agent需新幂等键才创建不同任务。终态删除保留最小id/owner/key/state/sequence及删除事件/Outbox，不TTL/自动抹除；不是完整Event Sourcing。

重复删除仅expected_sequence等于删除前或当前时返回既有deleted回执，不增事件；其他旧序号-32011。非终态及Busy/Unknown-32012；权限-32003、不可见-32004；迁移失败全部回滚，错误格式/未知版本拒绝。现有schema2→3迁移后同事务再到4，旧数据保留。无需新依赖/密钥/加密，AD-ST-01保持。

验证：实际文件迁移、四表删除/回滚、序号/身份/活动状态、重复删除与创建不复活；原生卡片删除打开确认后取消，不代替用户确认清理真实任务。实际删除使用独立测试库；macOS截图/日志，Windows仍暂缓，完整TM-S5与接管需求保留。
