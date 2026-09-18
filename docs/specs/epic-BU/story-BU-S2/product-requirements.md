# BU-S2 产品需求

## 问题与目标

Browser Task Space 目前只在单次Bridge返回中存在，Yonda重启或进入下一步骤后无法从任务事实源恢复引用。本Story把引用与已Observe attempt绑定，供任务展示、接管和后续Agent动作读取。

## 范围与非目标

保存外部引用、所有权、托管页数量、完成标记和更新时间序号；不保存页面正文、截图、浏览历史或ego-lite内部状态，不复制Browser Task Space。

## 验收条件

- 成功Bridge结果、attempt observed、任务sequence、事件、Outbox和浏览器引用同事务提交。
- unknown、身份不符、旧sequence、非法引用或Outbox失败不得更新引用。
- finish保留引用审计并标记完成；重启后可读，不自动重开或接管空间。
- Windows Runtime暂缓时明确能力不可用，不伪造引用。
- 授权Task Space可只读查看已提交引用的安全摘要；查询不得启动、接管或交接ego-lite空间。
- 2026-09-18用户修订：活动引用提供“打开 ego-lite”；用户点击后以`handOff()`进入对应Task Space，Agent恢复时以`takeOverTaskSpace()`进入同一空间。动作必须Observe并进入任务事件，不创建替代空间。

## 待决事项

Agent Gateway动作方法在引用事务通过后独立接入；Windows继续暂缓。

## 需求来源

- 产品简报「MVP主干链路」「Task Space与权限模型」。
- 补充材料「CUA与BUA的Task Space」「执行原则」。
- 后续用户要求连接本地Agent并集成CUA/BUA能力。
- 架构约束：ARCHITECTURE-SPINE、Accepted AD-E0-03、AD-TM-08与AD-BU-01。
