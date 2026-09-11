# OOXML Adapter 对照 Spike

生成不依赖候选实现的确定性样本：

```bash
python3 generate_fixtures.py
```

`fixtures/synthetic` 用于自动回归；WPS 实际保存的样本放入 `fixtures/wps`，不得用合成样本替代兼容性门禁。

下载并校验 Microsoft Open XML SDK 官方复杂样本：

```bash
sh fetch_real_fixtures.sh
```
