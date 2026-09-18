use std::{fs::{self,File,OpenOptions},io::Write,os::unix::fs::{MetadataExt,symlink},path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH}};

fn identity(path:&Path)->(u64,u64){let m=fs::metadata(path).unwrap();(m.dev(),m.ino())}
fn output_path(root:&Path,path:&Path)->Option<PathBuf>{
    if !path.is_absolute(){return None} let parent=path.parent()?.canonicalize().ok()?;let root=root.canonicalize().ok()?;
    parent.strip_prefix(&root).ok()?;Some(parent.join(path.file_name()?))
}

fn main(){
    let root=std::env::temp_dir().join(format!("yonda-file-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&root).unwrap();let original=root.join("source.txt");fs::write(&original,b"before").unwrap();
    let hard=root.join("hard.txt");fs::hard_link(&original,&hard).unwrap();let soft=root.join("soft.txt");symlink(&original,&soft).unwrap();
    assert_eq!(identity(&original),identity(&hard));assert_eq!(identity(&original),identity(&soft));

    let first=OpenOptions::new().read(true).write(true).open(&original).unwrap();first.try_lock().unwrap();
    let second=OpenOptions::new().write(true).open(&hard).unwrap();assert!(second.try_lock().is_err());

    let output=output_path(&root,&root.join("output.txt")).unwrap();assert!(output_path(&root,&Path::new("../relative")).is_none());
    let outside=root.join("escape");symlink("/tmp",&outside).unwrap();assert!(output_path(&root,&outside.join("escaped.txt")).is_none());
    let temporary=root.join(".output.tmp");let mut file=File::create(&temporary).unwrap();file.write_all(b"after").unwrap();file.sync_all().unwrap();drop(file);fs::rename(&temporary,&output).unwrap();assert_eq!(fs::read(&output).unwrap(),b"after");
    drop(second);drop(first);fs::remove_dir_all(&root).unwrap();
    println!("{}",r#"{"hardlink_same":true,"symlink_same":true,"second_lock_rejected":true,"escape_rejected":true,"atomic_commit":true,"passed":true}"#);
}
