# 全量运行状态查询

Story: TM-S6

关联 docs/specs/epic-TM/story-TM-S6/README.md；依据 AD-TM-02-RUNNING-STATE-QUERY.md。

分页或单一 Agent 的空列表不能说明全局没有执行任务。提供可信宿主只读查询，错误返回未知。

Architecture Impact: conforming

仅 Application 用例及 Adapter 集成测试；复用 TaskStore，无协议、迁移、依赖方向或密钥变更。桌面接线与完整隐藏门禁另行实施。

本次续作纳入 TM-S6 AC5–8：复用唯一 Admission 汇总未停止执行占用，未就绪/不可读保持 Unknown，避免终态早于资源释放时误报无工作。仍不接入桌面或授予隐藏许可。

续作 TM-S6 AC9–11：唯一 Admission 内协调收起预约与准入；不新增传输接口、原生接线或后台队列。依据 AD-TM-02 收起预约补充。
