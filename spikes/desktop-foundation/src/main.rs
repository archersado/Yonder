use std::{env, io};

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("server") => yonder_ipc_spike::serve_once().await,
        Some("client") => yonder_ipc_spike::call(args.next().unwrap_or_else(|| "hello".into())).await,
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "用法: yonder-ipc-spike server | client [payload]",
        )),
    }
}
