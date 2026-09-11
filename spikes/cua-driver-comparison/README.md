# CUA Driver 对照 Spike

固定版本的无 Agent 黑盒 Harness。探针只读取 Driver 元数据、工具清单和应用列表，不产生键鼠输入。

```bash
npm install
npm run probe:qwen
npm run probe:trycua
npm run fault:qwen
npm run fault:trycua
```

原始响应可能包含本机应用信息，保存在已忽略的 `evidence/`，不得提交仓库。
