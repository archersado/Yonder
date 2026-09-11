fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a != "--check") || args.len() > 1 { return Err("用法：generate [--check]".into()); }
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("generated");
    if args.is_empty() { std::fs::create_dir_all(&directory)?; }
    for (name, expected) in yonder_protocol::generated_artifacts() {
        let path = directory.join(name);
        if args.is_empty() { std::fs::write(path, expected)?; }
        else if std::fs::read_to_string(path)? != expected { return Err(format!("生成产物过期：{name}").into()); }
    }
    Ok(())
}
