# AD-BU-01 Browser Task Space 引用事实

状态：Accepted；日期：2026-09-16。关联BU-S2、TM-S1/TM-S2、AG-S1、DS-S2。

## 来源与决定

产品简报要求BUA直接复用ego-lite Task Space并在统一任务中展示、交接和接管；用户要求本地Agent连接。Yonder必须保存稳定外部引用，但不得复制浏览器页面状态。

Application以SQLite当前表持有每任务最新Browser引用：`external_task_ref`、ego-lite所有权、托管页数量、finished和updated_sequence。只有受监督Bridge成功并完成后置Observe后，引用才能与attempt observed、任务sequence、事件和Outbox在同一事务提交。unknown或失败不更新当前引用。

引用格式当前仅接受`ego:<正整数>`；Agent/UI不能自报或改写。finish保留引用并标记完成，不自动删除Task Space历史；重启只读取事实，不凭引用自动takeOver。正文、URL、标签标题、截图和浏览历史不存储。

## 兼容与迁移

schema11→12新增`task_browser_refs`，不重建任务或attempt表；迁移前备份。Windows没有Runtime时不写伪造行并保持capability unavailable。Gateway动作和UI入口在本事务验证后另行实施。
