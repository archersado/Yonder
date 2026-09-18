//! 独立目录测试宿主，复用正式桌面的私有stdio研发连接。
use std::{path::PathBuf, sync::{Arc, Mutex}};
use yonder_desktop::{TaskHost, local_agent_stdio};
fn main() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let directory = PathBuf::from(std::env::args_os().nth(1).ok_or("缺少测试目录")?);
        let host = TaskHost::open(&directory).map_err(|_| "测试宿主打开失败")?;
        local_agent_stdio::serve(Arc::new(Mutex::new(Some(host))))
    })();
    if result.is_err() { eprintln!("本地Agent测试连接失败"); std::process::exit(1); }
}
