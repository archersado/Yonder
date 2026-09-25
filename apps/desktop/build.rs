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
    // 内嵌资源变化也必须重新展开generate_context，避免预览仍运行旧脚本。
    println!("cargo:rerun-if-changed=ui");
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=src/voice_macos.m");
        println!("cargo:rerun-if-changed=src/region_capture_macos.m");
        cc::Build::new().file("src/voice_macos.m").flag("-fblocks").compile("yonda_voice");
        cc::Build::new().file("src/region_capture_macos.m").flag("-fblocks").compile("yonda_region_capture");
        for framework in ["AVFoundation", "Foundation", "Speech", "AppKit", "ApplicationServices", "ImageIO", "ScreenCaptureKit"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
    tauri_build::build()
}
