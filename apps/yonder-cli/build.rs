fn main() {
    println!("cargo:rerun-if-env-changed=YONDER_BUILD_COMMIT");
    let profile = std::env::var("PROFILE").expect("Cargo必须提供PROFILE");
    let commit = std::env::var("YONDER_BUILD_COMMIT").unwrap_or_else(|_| "development".to_owned());
    if profile == "release"
        && (commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        panic!("release构建必须通过YONDER_BUILD_COMMIT提供40位Git SHA");
    }
    println!("cargo:rustc-env=YONDER_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=YONDER_BUILD_PROFILE={profile}");
}
