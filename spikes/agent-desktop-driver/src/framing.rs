use std::io::{self, BufRead, Write};

use serde::{Serialize, de::DeserializeOwned};

use crate::protocol::MAX_FRAME_BYTES;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    Io,
    Oversize,
    PartialEof,
    InvalidUtf8,
    InvalidJson,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrameWriteError {
    BeforeWrite(FrameError),
    DeliveryUnknown(FrameError),
}

impl std::fmt::Display for FrameWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for FrameWriteError {}

impl std::fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for FrameError {}

pub fn read_frame<T: DeserializeOwned>(reader: &mut impl BufRead) -> Result<Option<T>, FrameError> {
    let mut frame = Vec::with_capacity(1024);
    let mut limited = std::io::Read::take(reader, (MAX_FRAME_BYTES + 1) as u64);
    let read = limited
        .read_until(b'\n', &mut frame)
        .map_err(|_| FrameError::Io)?;
    if read == 0 {
        return Ok(None);
    }
    if frame.len() > MAX_FRAME_BYTES {
        return Err(FrameError::Oversize);
    }
    if frame.last() != Some(&b'\n') {
        return Err(FrameError::PartialEof);
    }
    frame.pop();
    let text = std::str::from_utf8(&frame).map_err(|_| FrameError::InvalidUtf8)?;
    serde_json::from_str(text)
        .map(Some)
        .map_err(|_| FrameError::InvalidJson)
}

/// 返回时完整 JSONL 帧已经写入并 flush；这就是唯一派发线性化点。
pub fn write_frame<T: Serialize>(
    writer: &mut impl Write,
    value: &T,
) -> Result<(), FrameWriteError> {
    let mut frame = serde_json::to_vec(value)
        .map_err(|_| FrameWriteError::BeforeWrite(FrameError::InvalidJson))?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err(FrameWriteError::BeforeWrite(FrameError::Oversize));
    }
    writer
        .write_all(&frame)
        .map_err(|_| FrameWriteError::DeliveryUnknown(FrameError::Io))?;
    writer
        .flush()
        .map_err(|_| FrameWriteError::DeliveryUnknown(FrameError::Io))
}

impl From<FrameError> for io::Error {
    fn from(value: FrameError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, format!("{value:?}"))
    }
}

impl From<FrameWriteError> for io::Error {
    fn from(value: FrameWriteError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, format!("{value:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::io::{BufReader, Cursor};

    #[test]
    fn accepts_exact_limit_and_rejects_limit_plus_one_before_json_parse() {
        let prefix = b"{\"x\":\"";
        let suffix = b"\"}\n";
        let mut exact = Vec::from(prefix.as_slice());
        exact.extend(vec![b'a'; MAX_FRAME_BYTES - prefix.len() - suffix.len()]);
        exact.extend(suffix);
        assert!(read_frame::<Value>(&mut BufReader::new(Cursor::new(exact))).is_ok());
        let too_large = vec![b' '; MAX_FRAME_BYTES + 1];
        assert_eq!(
            read_frame::<Value>(&mut BufReader::new(Cursor::new(too_large))),
            Err(FrameError::Oversize)
        );
    }

    #[test]
    fn rejects_partial_invalid_utf8_and_invalid_json() {
        for (bytes, expected) in [
            (b"{}".to_vec(), FrameError::PartialEof),
            (vec![0xff, b'\n'], FrameError::InvalidUtf8),
            (b"not-json\n".to_vec(), FrameError::InvalidJson),
        ] {
            assert_eq!(
                read_frame::<Value>(&mut BufReader::new(Cursor::new(bytes))),
                Err(expected)
            );
        }
    }
}
