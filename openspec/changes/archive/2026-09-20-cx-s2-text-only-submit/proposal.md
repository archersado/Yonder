# Proposal：CX-S2 无截图文字提交

关联Story：CX-S2；关联决策：Accepted AD-CX-01、AD-CX-02、AD-VI-02。

## Why

当前无效小选区会清空页面但留下选择会话，截图权限缺失则确认卡无法发送，与CX2-06及既有“仅提问”设计不符。

## What Changes

让直接点击或不足12点的选择进入无附件确认卡；截图权限缺失时保留同一卡片。非空文字复用当前AgentSession的`agent.input`提交，不要求附件能力，不生成附件、不创建任务、不持久化正文。macOS正式bundle验证接受、拒绝、unknown、无附件能力与权限缺失清场；Windows继续暂缓。

Architecture Impact：conforming。只扩展现有Preview状态与Desktop组合，不改变协议、存储、依赖方向或状态所有者。
