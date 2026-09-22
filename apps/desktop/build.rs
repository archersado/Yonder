fn main() {
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
