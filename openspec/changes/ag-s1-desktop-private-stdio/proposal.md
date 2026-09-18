# AG-S1 正式宿主私有stdio联调

关联docs/specs/epic-AG/story-AG-S1/三份设计及Accepted AD-AG-03。Architecture Impact：architecture-change；仅正式桌面研发连接生命周期，不变更wire/schema/依赖方向。复用既有stdio夹具读帧实现与唯一TaskHost，让本地测试Agent登记任务后原生检查小龙菜单；不开放生产Socket/人工创建。
