# Epic DS：桌面宿主与桌宠

Epic: DS

## 模块边界

透明常驻窗口、桌宠生命周期、任务总览展示；不拥有任务状态。

## Stories

- [DS-S1 透明桌宠与桌面基础验证](story-DS-S1/README.md)
- [DS-S2 统一任务空间](story-DS-S2/README.md)
- [DS-S3 小龙环绕图标菜单](story-DS-S3/README.md)
- [DS-S4 桌宠动画资源包导入](story-DS-S4/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
