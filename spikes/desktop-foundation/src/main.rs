use interprocess::local_socket::{
    ListenerOptions,
    tokio::{Stream, prelude::*},
};
use serde::{Deserialize, Serialize};
use std::{env, io};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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

async fn serve_once() -> io::Result<()> {
    let listener = ListenerOptions::new()
        .name(socket_name()?)
        .try_overwrite(true)
        .create_tokio()?;
    let stream = listener.accept().await?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).await?;
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

async fn call(payload: String) -> io::Result<()> {
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("server") => serve_once().await,
        Some("client") => call(args.next().unwrap_or_else(|| "hello".into())).await,
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "用法: yonder-ipc-spike server | client [payload]",
        )),
    }
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
}
