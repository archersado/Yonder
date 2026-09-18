use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
mod pet_window;

#[tauri::command]
fn pet_is_visible(window: tauri::WebviewWindow) -> Result<bool, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    Ok(window.is_visible().map_err(|_| "无法读取可见性")?
        && !window.is_minimized().map_err(|_| "无法读取最小化状态")?)
}

#[tauri::command]
fn report_pet_motion(window: tauri::WebviewWindow, breathing: bool, blinks: u32, reduced: bool) {
    if window.label() == "pet" {
        eprintln!("桌宠动画采样（14秒）：breathing={breathing}, blinks={blinks}, reduced={reduced}");
    }
}

#[tauri::command]
fn report_pet_render(window: tauri::WebviewWindow, ready: bool, width: f64, height: f64, image_width: u32, style_loaded: bool, script_ready: bool, page_hidden: bool) {
    if window.label() == "pet" {
        eprintln!("桌宠渲染：ready={ready}, width={width}, height={height}, image_width={image_width}, style_loaded={style_loaded}, script_ready={script_ready}, page_hidden={page_hidden}");
    }
}

fn main() {
    tauri::Builder::default()
        .manage(pet_window::PetWindowState::default())
        .invoke_handler(tauri::generate_handler![report_pet_render, report_pet_motion, pet_is_visible, pet_window::pet_dock, pet_window::pet_wake])
        .setup(|app| {
            // 仅验证同进程Socket，绝不暴露任务或其他执行能力。
            app.manage(std::sync::Mutex::new(yonder_ipc_spike::ProbeServer::start()?));
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let pet = app.get_webview_window("pet").ok_or("桌宠窗口未创建")?;
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior as Behavior};
                // setup 在主线程运行；原生窗口由 pet 持有，借用不跨此作用域。
                let native = unsafe { &*pet.ns_window()?.cast::<NSWindow>() };
                let mut behavior = native.collectionBehavior();
                if objc2::available!(macos = 13.0) {
                    behavior = (behavior & !(Behavior::Primary | Behavior::Auxiliary))
                        | Behavior::CanJoinAllApplications;
                }
                native.setCollectionBehavior(
                    (behavior & !(Behavior::FullScreenPrimary | Behavior::FullScreenNone | Behavior::FullScreenAllowsTiling))
                        | Behavior::FullScreenAuxiliary | Behavior::FullScreenDisallowsTiling,
                );
            }
            if let Some(monitor) = pet.primary_monitor()? {
                let origin = monitor.position();
                let screen = monitor.size();
                let size = pet.outer_size()?;
                pet.set_position(tauri::PhysicalPosition::new(
                    origin.x + (screen.width.saturating_sub(size.width) / 2) as i32,
                    origin.y + (screen.height.saturating_sub(size.height) / 2) as i32,
                ))?;
            } else {
                pet.center()?;
            }
            pet.show()?;
            pet.set_focus()?;
            eprintln!("桌宠启动：visible={:?}, position={:?}, size={:?}", pet.is_visible(), pet.outer_position(), pet.inner_size());
            let show = MenuItem::with_id(app, "show-pet", "显示小龙", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出 Yonda", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            // 原生模板图标 Y，无额外图片解码依赖。
            let mut rgba = vec![0u8; 16 * 16 * 4];
            for y in 2..14usize {
                for x in 2..14usize {
                    if (y < 8 && ((x as isize - y as isize).abs() <= 1 || (x + y).abs_diff(15) <= 1))
                        || (y >= 7 && (7..=8).contains(&x)) {
                        rgba[(y * 16 + x) * 4 + 3] = 255;
                    }
                }
            }
            TrayIconBuilder::with_id("e0-tray")
                .icon(tauri::image::Image::new_owned(rgba, 16, 16))
                .icon_as_template(true)
                .tooltip("Yonda")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show-pet" => {
                        if let Some(pet) = app.get_webview_window("pet") {
                            let result = pet.unminimize().and_then(|_| pet.show())
                                .and_then(|_| pet.set_focus())
                                .and_then(|_| pet.eval("window.dispatchEvent(new Event('yonda-show'))"));
                            if result.is_err() { eprintln!("托盘找回小龙失败"); }
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_page_load(|webview, payload| {
            eprintln!("桌宠页面加载：{:?}", payload.event());
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) {
                let _ = webview.eval(r#"
                    setTimeout(async () => {
                      const image = document.querySelector('.dragon');
                      const pet = document.querySelector('#pet');
                      let ready = false;
                      try { await image.decode(); ready = true; } catch (_) {}
                      const rect = image?.getBoundingClientRect();
                      window.__TAURI_INTERNALS__.invoke('report_pet_render', {
                        ready, width: rect?.width ?? 0, height: rect?.height ?? 0,
                        imageWidth: image?.naturalWidth ?? 0,
                        styleLoaded: !!pet && getComputedStyle(pet).paddingTop === '6px',
                        scriptReady: !!pet?.dataset.mode, pageHidden: document.hidden
                      });
                      // 每次有界采样，不记录内容或建立常驻轮询；重复触发合并。
                      const breath = document.querySelector('.breath');
                      const preference = matchMedia('(prefers-reduced-motion: reduce)');
                      let sampling = false;
                      function sampleMotion() {
                        if (sampling) return;
                        sampling = true;
                        let transform = getComputedStyle(breath).transform;
                        let breathing = false, blinks = 0;
                        let previous = pet.classList.contains('blinking');
                        const observer = new MutationObserver(() => {
                          const current = getComputedStyle(breath).transform;
                          breathing ||= current !== transform;
                          transform = current;
                          const closed = pet.classList.contains('blinking');
                          if (closed && !previous) blinks++;
                          previous = closed;
                        });
                        observer.observe(pet, {attributes: true, attributeFilter: ['class']});
                        observer.observe(breath, {attributes: true, attributeFilter: ['style']});
                        setTimeout(() => {
                          observer.disconnect();
                          sampling = false;
                          window.__TAURI_INTERNALS__.invoke('report_pet_motion', {
                            breathing, blinks, reduced: preference.matches
                          });
                        }, 14000);
                      }
                      window.addEventListener('yonda-show', sampleMotion);
                      preference.addEventListener('change', sampleMotion);
                      sampleMotion();
                    }, 300);
                "#);
            }
        })
        .build(tauri::generate_context!())
        .expect("Yonder E0 启动失败")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                if let Ok(mut probe) = app.state::<std::sync::Mutex<yonder_ipc_spike::ProbeServer>>().lock() {
                    probe.stop();
                }
            }
        });
}
