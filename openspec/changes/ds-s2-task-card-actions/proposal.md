# DS-S2 任务卡片操作入口

关联docs/specs/epic-DS/story-DS-S2/三份设计及Accepted AD-TM-03/04/06。Architecture Impact：conforming（卡片布局）；取消直接置于卡片，用户明确“删除”只取消且数据保留，统一取消任务入口。接管依赖停止确认/可见用户记录，未接通禁用并说明，不伪报控制移交。任务详情与操作为兄弟按钮，保留键盘焦点；删除清理实验已撤销。协议/实验格式兼容由关联tm-s3-cancel-retain-data承接。

## Why

任务卡片需要直接提供用户可理解的接管和取消入口，同时不能把未满足停止、定位或Recording门禁的操作伪装成已接管。

## What Changes

每张卡片提供“接管”和“取消任务”两个兄弟按钮；created任务可取消，执行中任务在停止确认完成后进入接管定位流程。停止、定位或Recording前置不满足时必须禁用并给出原因；取消保留全部数据和历史。协议与实验格式兼容由`tm-s3-cancel-retain-data`承接，接管停止与定位事实由`tm-s3-takeover-work-focus`承接。
