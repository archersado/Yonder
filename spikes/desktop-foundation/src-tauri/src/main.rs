use tauri::tray::TrayIconBuilder;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            TrayIconBuilder::with_id("e0-tray")
                .tooltip("Yonder E0")
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Yonder E0 启动失败");
}
