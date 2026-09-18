# Epic CU：Computer Use Driver

Epic: CU

## 模块边界

模型无关桌面动作、Observe、停止及用户输入接管。

## Stories

- [CU-S1 桌面 Driver 技术选型](story-CU-S1/README.md)
- [CU-S2 受监管桌面执行与 Observe](story-CU-S2/README.md)
- [CU-S3 后台原生动作与显式前台切换](story-CU-S3/README.md)

## 验收与依赖

按每个 Story 验收；不以库层测试代替原生双平台闭环。跨模块依赖只通过既有 Port/Application/Protocol，不因 Epic 划分改变 Cargo 依赖方向。
