#[cfg(windows)]
mod windows_probe {
    use std::{fs::OpenOptions, io, path::Path};

    fn occupied(path: &Path) -> io::Result<bool> {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => match file.try_lock() {
                Ok(()) => Ok(false),
                Err(_) => Ok(true),
            },
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(true),
            Err(error) => Err(error),
        }
    }

    pub fn run() -> io::Result<()> {
        let mut arguments = std::env::args_os().skip(1);
        let path = arguments
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "缺少隔离文档路径"))?;
        let expected = arguments
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "缺少open或closed"))?;
        let host = arguments
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "缺少office或wps"))?;
        if !matches!(expected.as_str(), "open" | "closed")
            || !matches!(host.as_str(), "office" | "wps")
            || arguments.next().is_some()
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "参数无效"));
        }
        let path = Path::new(&path);
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "文档路径必须为绝对路径",
            ));
        }
        let path = path.canonicalize()?;
        let occupied = occupied(&path)?;
        let expected_open = expected == "open";
        if occupied != expected_open {
            return Err(io::Error::new(io::ErrorKind::Other, "宿主锁状态与预期不符"));
        }
        println!(
            r#"{{"platform":"windows","host":"{host}","expected_open":{expected_open},"write_lock_rejected":{occupied},"passed":true}}"#
        );
        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_probe::run() {
        println!(
            r#"{{"platform":"windows","passed":false,"error_class":"{:?}"}}"#,
            error.kind()
        );
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("windows-office-lock-probe只能在Windows实机生成验证证据");
    std::process::exit(2);
}
