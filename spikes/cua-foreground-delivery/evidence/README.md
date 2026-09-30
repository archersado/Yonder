# CU-S4 macOS结构化证据

- `schema-0.30.4.json`：候选工具目录字段布尔。
- `baseline-0.25.0.json`：正式固定版本统一正向样本。
- `candidate-0.30.4.json`：升级候选统一正向样本。
- `recovery-failure-0.25.0.json`：原前台失效时阻断成功的负向样本。

证据不含截图、窗口标题、PID、window id、固定标记正文或完整SDK Payload。`verify.py`复核后结论为保留0.25.0；0.30.4未进入产品依赖。
