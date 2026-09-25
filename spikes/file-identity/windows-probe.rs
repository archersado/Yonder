#[cfg(windows)]
mod windows_probe {
    use std::{
        ffi::c_void,
        fs::{self, File, OpenOptions},
        io::{self, Write},
        os::windows::{
            fs::{symlink_dir, symlink_file},
            io::AsRawHandle,
        },
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct FileInformation {
        attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            handle: *mut c_void,
            information: *mut FileInformation,
        ) -> i32;
    }

    fn identity(path: &Path) -> io::Result<(u32, u64)> {
        let file = File::open(path)?;
        let mut information = FileInformation::default();
        let succeeded =
            unsafe { GetFileInformationByHandle(file.as_raw_handle().cast(), &mut information) };
        if succeeded == 0 {
            return Err(io::Error::last_os_error());
        }
        let file_index =
            (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
        Ok((information.volume_serial_number, file_index))
    }

    fn output_path(root: &Path, path: &Path) -> Option<PathBuf> {
        if !path.is_absolute() {
            return None;
        }
        let parent = path.parent()?.canonicalize().ok()?;
        let root = root.canonicalize().ok()?;
        parent.strip_prefix(&root).ok()?;
        Some(parent.join(path.file_name()?))
    }

    fn sample(root: &Path, outside: &Path) -> io::Result<()> {
        fs::create_dir(root)?;
        fs::create_dir(outside)?;
        let original = root.join("source.txt");
        fs::write(&original, b"before")?;
        let hard = root.join("hard.txt");
        fs::hard_link(&original, &hard)?;
        let soft = root.join("soft.txt");
        symlink_file(&original, &soft).map_err(|error| {
            io::Error::new(
                error.kind(),
                "无法创建软链接夹具；请启用Windows Developer Mode或使用具备符号链接权限的终端",
            )
        })?;

        let original_identity = identity(&original)?;
        if identity(&hard)? != original_identity || identity(&soft)? != original_identity {
            return Err(io::Error::new(io::ErrorKind::Other, "文件别名身份未归并"));
        }

        let first = OpenOptions::new().read(true).write(true).open(&original)?;
        first.try_lock()?;
        let second = OpenOptions::new().write(true).open(&hard)?;
        if second.try_lock().is_ok() {
            return Err(io::Error::new(io::ErrorKind::Other, "同一文件取得第二写锁"));
        }

        let output = output_path(root, &root.join("output.txt"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "受控输出被错误拒绝"))?;
        if output_path(root, Path::new("..\\relative.txt")).is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "相对路径未被拒绝"));
        }
        let escape = root.join("escape");
        symlink_dir(outside, &escape)?;
        if output_path(root, &escape.join("escaped.txt")).is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "链接逃逸未被拒绝"));
        }

        let temporary = root.join(".output.tmp");
        let mut file = File::create(&temporary)?;
        file.write_all(b"after")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &output)?;
        if fs::read(&output)? != b"after" {
            return Err(io::Error::new(io::ErrorKind::Other, "提交结果不一致"));
        }
        drop(second);
        drop(first);
        Ok(())
    }

    pub fn run() -> io::Result<()> {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos()
        );
        let root = std::env::temp_dir().join(format!("yonda-file-{unique}"));
        let outside = std::env::temp_dir().join(format!("yonda-file-outside-{unique}"));
        let result = sample(&root, &outside);
        let root_cleanup = fs::remove_dir_all(&root);
        let outside_cleanup = fs::remove_dir_all(&outside);
        result?;
        root_cleanup?;
        outside_cleanup?;
        println!(
            "{}",
            r#"{"platform":"windows","identity":"volume_serial_and_file_index","hardlink_same":true,"symlink_same":true,"second_lock_rejected":true,"escape_rejected":true,"same_directory_commit":true,"passed":true}"#
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
    eprintln!("windows-probe只能在Windows实机生成验证证据");
    std::process::exit(2);
}
