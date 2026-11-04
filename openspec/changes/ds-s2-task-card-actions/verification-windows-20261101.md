# Windows 卡片 created 取消子范围

结论：PASS。原生任务卡片把真实 `created` 任务取消为 `cancelled`，刷新后任务与两条事件仍可读；详见 [合并 Windows Verification Goal](../ds-s2-running-pagination/verification-windows-20261101.md)。

本结论不覆盖执行中停止、接管定位、Recording 或交回闸，因此完整 Change 不 Archive。
