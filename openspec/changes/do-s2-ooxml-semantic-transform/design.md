# 设计

Application提供格式、读取结果、唯一文本替换请求、稳定错误和`DocumentPort`。Adapter使用AD-E0-04已选定的`zip`、`quick-xml`与`sha2`，只扫描各格式正文文本part；调用方看不到part名称。

替换必须恰好命中一个XML文本节点。输出重新打开并验证格式标志和修改后的XML；输入字节不可变。文件安全与副作用由后续FI-S1组合层负责。
