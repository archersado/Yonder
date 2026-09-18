# 设计

Application定义WorkRef、WorkFocusPort及稳定结果分类。macOS Adapter把原生实现直接链接进Rust库，保留AX对象；不运行脚本、辅助App或sidecar。捕获只接受可信attempt和WorkTarget；定位复核权限、进程启动时间、PID/窗口ID、标题/几何、保留对象成员关系与唯一映射，再恢复最小化并前置，最后读取新鲜焦点和几何。引用释放或宿主重启后失效。
