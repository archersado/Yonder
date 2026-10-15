# 设计：前端协议门禁与 Desktop 验证目录

## 前端单一来源门禁

- `check_frontend_single_source(root)` 只扫描 `apps/desktop/ui/*.js`，不解释或执行页面代码。
- 页面一旦包含协议状态字面量集合或以协议字段为键的本地映射，就必须包含精确来源声明 `crates/protocol/generated/protocol.ts`；声明的目标文件必须存在。
- 门禁允许把 wire 值映射为本地展示文案，但禁止声明第二套可独立演进的状态枚举来源。检测保持有界、确定性，并以合成目录测试接受和拒绝样本。
- `check_architecture.py` 的正式入口必须调用该检查；错误消息给出页面相对路径和缺失约束。

## 验证脚本目录

- 所有 `apps/desktop/check-*` 文件机械迁移至 `scripts/verification/desktop/`，文件名和内容用途保持不变。
- Python 脚本的仓库根推导从新目录层级计算；脚本间引用与 Node `new URL` 相对路径同步到新位置。
- 全仓受版本控制文本中的 `apps/desktop/check-` 引用统一替换为新路径；迁移完成后架构门禁断言 Desktop 根目录不存在 `check-*`，新目录脚本数与迁移清单一致。
- 不移动 `apps/desktop/tests`、fixture、打包脚本、证据目录或产品资源。

## 验证

依次运行架构门禁自测、正式架构检查、Python 编译、Node 语法检查、Swift parse、代表性无外部副作用脚本、协议生成和 Workspace 测试。原生账户/权限型脚本只验证可解析性与路径解析，不在本 Change 中制造业务副作用。
