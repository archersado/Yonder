use interprocess::local_socket::{
    ListenerOptions,
    tokio::{Stream, prelude::*},
};
use serde::{Deserialize, Serialize};
use std::{env, io};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "0.1";

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Message {
    version: String,
    payload: String,
}

#[cfg(unix)]
fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    use interprocess::local_socket::GenericFilePath;
    env::temp_dir()
        .join("yonder-e0-desktop-foundation.sock")
        .to_fs_name::<GenericFilePath>()
}

#[cfg(windows)]
fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    use interprocess::local_socket::GenericNamespaced;
    "yonder-e0-desktop-foundation".to_ns_name::<GenericNamespaced>()
}

pub async fn serve_once() -> io::Result<()> {
    let listener = ListenerOptions::new()
        .name(socket_name()?)
        .try_overwrite(true)
        .create_tokio()?;
    exchange(listener.accept().await?).await
}

async fn exchange(stream: Stream) -> io::Result<()> {
    let mut line = String::new();
    BufReader::new(&stream).take(65_537).read_line(&mut line).await?;
    if line.len() > 65_536 { return Err(io::Error::new(io::ErrorKind::InvalidData, "验证请求过大")); }
    let request: Message = serde_json::from_str(&line).map_err(io::Error::other)?;
    if request.version != PROTOCOL_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "协议版本不匹配",
        ));
    }
    let mut response = serde_json::to_vec(&request).map_err(io::Error::other)?;
    response.push(b'\n');
    (&stream).write_all(&response).await
}

pub async fn call(payload: String) -> io::Result<()> {
    let stream = Stream::connect(socket_name()?).await?;
    let request = Message {
        version: PROTOCOL_VERSION.into(),
        payload,
    };
    let mut bytes = serde_json::to_vec(&request).map_err(io::Error::other)?;
    bytes.push(b'\n');
    (&stream).write_all(&bytes).await?;

    let mut response = String::new();
    BufReader::new(&stream).read_line(&mut response).await?;
    let echoed: Message = serde_json::from_str(&response).map_err(io::Error::other)?;
    if echoed != request {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "echo 不一致"));
    }
    println!("{response}", response = response.trim_end());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_round_trip() {
        let message = Message {
            version: PROTOCOL_VERSION.into(),
            payload: "由达".into(),
        };
        let encoded = serde_json::to_string(&message).unwrap();
        assert_eq!(serde_json::from_str::<Message>(&encoded).unwrap(), message);
    }

    #[tokio::test]
    async fn supervised_echo_rejection_and_stop() {
        let mut server = ProbeServer::start().unwrap();
        for request in [b"{\"version\":\"invalid\",\"payload\":\"probe\"}\n".to_vec(), vec![b'x'; 65_537]] {
            let stream = Stream::connect(socket_name().unwrap()).await.unwrap();
            (&stream).write_all(&request).await.unwrap();
            let mut byte = [0];
            assert_eq!((&stream).read(&mut byte).await.unwrap(), 0);
        }
        call("宿主验证".into()).await.unwrap();
        let _silent = Stream::connect(socket_name().unwrap()).await.unwrap();
        // 未发完整请求的连接也不能阻止宿主正常退出。
        server.stop();
        #[cfg(unix)]
        assert!(!env::temp_dir().join("yonder-e0-desktop-foundation.sock").exists());
    }
}

/// 仅供同进程桌面基础栈验证，不是生产Gateway或任务执行器。
pub struct ProbeServer {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ProbeServer {
    pub fn start() -> io::Result<Self> {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let thread = std::thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let runtime = match result {
                Ok(runtime) => runtime,
                Err(error) => { let _ = ready_tx.send(Err(error)); return; }
            };
            runtime.block_on(async {
                let listener = match socket_name().and_then(|name| ListenerOptions::new().name(name).create_tokio()) {
                    Ok(listener) => listener,
                    Err(error) => { let _ = ready_tx.send(Err(error)); return; }
                };
                if ready_tx.send(Ok(())).is_err() { return; }
                tokio::select! {
                    _ = stopped => {},
                    _ = async {
                        while let Ok(stream) = listener.accept().await {
                            // 错误输入只关闭该连接；不记录请求内容。
                            let _ = exchange(stream).await;
                        }
                    } => {}
                }
                // listener和未完成连接随验证future退出而释放。
            });
        });
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self { stop: Some(stop), thread: Some(thread) }),
            result => {
                let _ = thread.join();
                Err(match result {
                    Ok(Err(error)) => error,
                    _ => io::Error::other("验证线程初始化中断"),
                })
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(stop) = self.stop.take() { let _ = stop.send(()); }
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

impl Drop for ProbeServer {
    fn drop(&mut self) { self.stop(); }
}
