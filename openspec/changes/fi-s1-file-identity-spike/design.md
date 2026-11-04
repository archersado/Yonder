# 设计

macOS样本以标准库`MetadataExt`取得`dev/ino`，分别通过原路径、硬链接和软链接访问同一文件；以规范化父目录判断未创建输出是否仍在授权根；同目录临时文件写完后rename。锁样本只验证Yonder实例间的advisory lock，Office/WPS真实锁另行取证。

Windows用volume serial/file index和同一断言复验。文件符号链接受开发者模式/管理员权限限制时，统一样本使用无需提权的目录 Junction 形成指向同一文件的 reparse 别名，并同时记录文件符号链接不可创建的环境原因；硬链接和宿主锁断言不变。双平台与宿主锁通过前不接受AD。
