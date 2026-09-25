# 设计

macOS样本以标准库`MetadataExt`取得`dev/ino`，分别通过原路径、硬链接和软链接访问同一文件；以规范化父目录判断未创建输出是否仍在授权根；同目录临时文件写完后rename。锁样本只验证Yonder实例间的advisory lock，Office/WPS真实锁另行取证。

Windows使用文件句柄的volume serial/file index归并原路径、硬链接与软链接，并复验同文件写锁、规范化父目录、链接逃逸和同目录提交。独立宿主锁探针只输出预期状态、是否检测到占用和通过标志，不输出文件路径或正文；由Windows实机在Office/WPS打开与关闭同一隔离DOCX时各运行一次。探针实现完成不等于实机证据通过；双平台与宿主锁通过前不接受AD。
