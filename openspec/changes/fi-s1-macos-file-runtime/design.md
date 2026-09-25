# 设计

Application 的 `FilePort` 接收绝对目标与绝对授权根。读取返回最多 16 MiB 的内容、平台无关身份和 SHA-256。写入分 `create-new` 与 `replace`：替换同时携带读取时身份和哈希；校验器只查看暂存字节，不获得任务库或用户身份。

macOS Adapter 先规范化授权根。既有文件 canonicalize 后必须仍位于根内，以 `st_dev + st_ino` 作为身份；新目标只接受单层已存在规范父目录，并以父目录身份与原始文件名字节建立占用键。进程内写租约先归并别名，再取得目标 advisory lock；两层任一冲突都不写入。

暂存文件在目标目录 `create_new`，写入后 `sync_all`，重新读取暂存字节交给校验器，通过后再次核验目标身份与哈希并原子 rename，最后同步父目录。rename 后父目录同步失败返回 unknown；其他失败删除暂存文件。非 macOS Adapter 只返回 `UnsupportedPlatform`。

回收站用例先由 Application 拒绝 Agent 身份并校验预期文件身份；macOS 调用系统 `NSFileManager.trashItem`。只有原路径消失且系统返回位置存在才确定成功；永久删除无 API。
