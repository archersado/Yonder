# 设计

Application新增`advance_after_observe`与`prepare_next_attempt`。SQLite schema11允许stopped attempt分为普通边界（control字段全空）和控制边界（control字段齐全）；普通推进、sequence、running→running事件、Outbox同事务。running步骤声明只接受最新普通stopped attempt且无pending控制；下一attempt准备同样原子追加事件，不重新取得或释放Permit。

本Change不开放Gateway动作，不持久化BUA外部引用，不改变控制停止、unknown或终态语义。
