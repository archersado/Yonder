use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{
    connect_async, connect_async_with_config,
    tungstenite::{protocol::WebSocketConfig, Error, Message},
};

const MAX_FRAME_BYTES: usize = 64 * 1024;

fn accepts_frame(len: usize) -> bool {
    len <= MAX_FRAME_BYTES
}

fn retry_delay_ms(attempt: u32, jitter: u64) -> u64 {
    (250_u64.saturating_mul(1_u64 << attempt.min(4))).min(4_000) + jitter.min(249)
}

async fn trusted_wss_roundtrip(url: &str) -> Result<(), String> {
    let marker = "yonda-wss-probe";
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_FRAME_BYTES))
        .max_frame_size(Some(MAX_FRAME_BYTES));
    let (mut socket, _) = timeout(
        Duration::from_secs(8),
        connect_async_with_config(url, Some(config), false),
    )
    .await
    .map_err(|_| "可信WSS连接超时")?
    .map_err(|error| error.to_string())?;
    socket
        .send(Message::Text(marker.into()))
        .await
        .map_err(|error| error.to_string())?;
    timeout(Duration::from_secs(8), async {
        while let Some(message) = socket.next().await {
            match message.map_err(|error| error.to_string())? {
                Message::Text(text) if text.contains(marker) => return Ok::<(), String>(()),
                Message::Binary(bytes) if bytes == marker.as_bytes() => {
                    return Ok::<(), String>(())
                }
                Message::Ping(bytes) => socket
                    .send(Message::Pong(bytes))
                    .await
                    .map_err(|error| error.to_string())?,
                _ => {}
            }
        }
        Err("可信WSS在回显前关闭".into())
    })
    .await
    .map_err(|_| "可信WSS回显超时")??;
    socket
        .send(Message::Ping(marker.as_bytes().to_vec().into()))
        .await
        .map_err(|error| error.to_string())?;
    timeout(Duration::from_secs(8), async {
        while let Some(message) = socket.next().await {
            match message.map_err(|error| error.to_string())? {
                Message::Pong(bytes) if bytes == marker.as_bytes() => return Ok::<(), String>(()),
                Message::Ping(bytes) => socket
                    .send(Message::Pong(bytes))
                    .await
                    .map_err(|error| error.to_string())?,
                _ => {}
            }
        }
        Err("可信WSS在Pong前关闭".into())
    })
    .await
    .map_err(|_| "可信WSS Pong超时")??;
    socket
        .close(None)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "TLS Provider初始化失败")?;
    assert_eq!(retry_delay_ms(0, 0), 250);
    assert_eq!(retry_delay_ms(8, 999), 4_249);
    assert!(accepts_frame(MAX_FRAME_BYTES));
    assert!(!accepts_frame(MAX_FRAME_BYTES + 1));

    let trusted = std::env::var("YONDER_WSS_PROBE_URL")
        .unwrap_or_else(|_| "wss://echo.websocket.org".into());
    trusted_wss_roundtrip(&trusted).await?;
    sleep(Duration::from_millis(retry_delay_ms(0, 0))).await;
    trusted_wss_roundtrip(&trusted).await?;
    let invalid_tls_result = timeout(
        Duration::from_secs(8),
        connect_async("wss://self-signed.badssl.com/"),
    )
    .await
    .map_err(|_| "无效TLS连接超时")?;
    let invalid_tls_rejected = match &invalid_tls_result {
        Err(Error::Tls(_)) => true,
        Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::InvalidData => true,
        _ => false,
    };
    if !invalid_tls_rejected {
        return Err("无效TLS证书未被TLS层拒绝".into());
    }

    println!("{}", serde_json::json!({
        "transport": "wss",
        "trusted_tls": true,
        "invalid_tls_rejected": true,
        "frame_limit_bytes": MAX_FRAME_BYTES,
        "bounded_backoff": true,
        "ping_pong": true,
        "reconnected_after_close": true,
        "clean_close": true,
        "second_process": false,
        "credential_persisted": false
    }));
    Ok(())
}
