# 设计

macOS样本以标准库`MetadataExt`取得`dev/ino`，分别通过原路径、硬链接和软链接访问同一文件；以规范化父目录判断未创建输出是否仍在授权根；同目录临时文件写完后rename。锁样本只验证Yonder实例间的advisory lock，Office/WPS真实锁另行取证。

Windows后续用volume serial/file index和同一断言复验。双平台与宿主锁通过前不接受AD。
