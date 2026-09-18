# 独立 Verification Goal

日期2026-09-14，macOS研发。关联DS-S2 CARD-01–04/TM-S5 DELETE-01–04及AD-TM-04/05，本Change规格。实现结束后独立验证，当前verifying。实际删除只在临时文件库，原生真实卡片打开确认后取消，不自动确认删除。Windows和执行中接管前置保留，完整Story不Archive。

当前目标已由用户澄清改为接管/取消卡片，不验证清理弹窗。首次原生入口exit4悬停未打开，清理请求未发出；当前真实库两任务及删除标记0已证实。保留失败证据，重新部署保留数据版本后验证。

## 最终结果

当前卡片入口首批PASS，完整DS-S2不Done/Archive。真实[原生结果](../../../apps/desktop/evidence/task-card-retain-outside-20260914/result.json)确认：接管/取消卡片按钮、尚未执行禁用接管、托盘全部可查已取消记录、刷新可操作、面板内保留、移出收起、当前同桌面。两任务已取消，hover不打开符合仅非终态自动面板规格；不据此恢复任务或插入假运行数据。[卡片截图](../../../apps/desktop/evidence/task-card-retain-outside-20260914/native-card-actions.png)。

前次脚本未覆盖任务全部取消场景，随后静态文本匹配过早/命中非按钮导致exit10，精确限定按钮角色及等待加载后验证入口通过；移出常量(20,100)未保证在窗口外导致exit9。最终使用CGWindow矩形选取两窗口外坐标后原生收起通过，UI未为通过断言改写状态。旧失败目录保留。

数据/协议撤销与安全兼容独立验证见tm-s3-cancel-retain-data/verification-retain-data.md；接管/Recording、执行中取消和Windows仍待实施验证，禁用原因明确，不将按钮存在宣称为功能接通。
