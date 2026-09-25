# TM-S5 控制请求与停止确认历史

Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」，TM5-AC02/07/08。决策：Accepted AD-TM-18。

让已提交的控制请求和步骤边界停止各自按真实事件序号可检查，避免单靠状态迁移猜测“已接管”。Architecture Impact：architecture-change（协议 1.22）；只读关联现有 SQLite 控制记录，不新增写入或迁移。定位、录制与交回不属于本 Change。
