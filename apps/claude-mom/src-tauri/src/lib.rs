//! ClaudeMom：用原版 Claude Desktop 的完整功能模式，直连自己配置的上游（Base URL + Key）。

mod commands;
mod host;
mod log;
mod upstreams;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

const MAIN_WINDOW: &str = "main";
const TRAY_SHOW: &str = "tray_show";
const TRAY_QUIT: &str = "tray_quit";
/// 单实例锁端口（与 Codex++ 的 57319 / 57320 错开）。设置 `CLAUDE_MOM_GUARD_PORT` 可覆盖。
const GUARD_PORT: u16 = 57519;

static EXITING: AtomicBool = AtomicBool::new(false);
/// 常驻实例的 AppHandle：唤醒线程拿到它才能触发启动。
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
/// 第二次从桌面图标启动时，由新进程发给常驻实例的请求。
const WAKE_LAUNCH: &str = "launch-claude";

pub fn run() {
    host::ClaudeMomHost::install();
    log::append("app.start", serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }));
    let launch_claude = std::env::args().any(|arg| arg == "--launch-claude");
    let guard_port = std::env::var("CLAUDE_MOM_GUARD_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(GUARD_PORT);
    // 已经有一个常驻实例：把「从桌面图标启动」的请求转给它，再自己退出。
    // 不转达的话，托盘常驻时点 Claude 图标会毫无反应（用户看到的就是「没生效」）。
    let listener = match TcpListener::bind(("127.0.0.1", guard_port)) {
        Ok(listener) => listener,
        Err(_) => {
            if launch_claude
                && let Ok(mut stream) = TcpStream::connect(("127.0.0.1", guard_port))
            {
                let _ = stream.write_all(WAKE_LAUNCH.as_bytes());
                let _ = stream.flush();
            }
            log::append("app.already_running", serde_json::json!({ "port": guard_port }));
            return;
        }
    };
    // 常驻实例：接收后来者的唤醒请求（第二次点桌面图标走这里）。
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let mut stream = stream;
                let mut buffer = [0u8; 64];
                let Ok(read) = stream.read(&mut buffer) else {
                    return;
                };
                if String::from_utf8_lossy(&buffer[..read]).contains(WAKE_LAUNCH)
                    && let Some(app) = APP.get()
                {
                    launch_claude_now(app.clone());
                }
            });
        }
    });
    // 程序重启后，Claude 可能还挂在旧代理端口上：在原端口把本地服务拉起来，它的重试就能连上。
    std::thread::spawn(|| match claude_local::resume_attached_proxy() {
        Ok(Some(port)) => log::append("claude_local.resumed", serde_json::json!({ "port": port })),
        Ok(None) => {}
        Err(error) => log::append("claude_local.resume_failed", serde_json::json!({ "message": format!("{error:#}") })),
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            install_tray(app)?;
            // 唤醒线程要用：注册当前实例，后来者才能触发启动。
            let _ = APP.set(app.handle().clone());
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let hide_target = window.clone();
                window.on_window_event(move |event| {
                    // 关窗只是收起：本地服务要一直在，Claude 才连得上。真正退出走托盘菜单。
                    if let WindowEvent::CloseRequested { api, .. } = event
                        && !EXITING.load(Ordering::SeqCst)
                    {
                        api.prevent_close();
                        let _ = hide_target.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::claude_local_status,
            commands::claude_local_configure,
            commands::claude_local_action,
            commands::claude_mom_load,
            commands::claude_mom_save_profile,
            commands::claude_mom_delete_profile,
            commands::claude_mom_set_active,
            commands::claude_mom_import,
            commands::claude_mom_fetch_models,
            commands::claude_mom_set_voice,
            commands::dictation_local_model_status,
            commands::download_dictation_local_model,
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            log::append("app.build_failed", serde_json::json!({ "message": error.to_string() }));
            return;
        }
    };
    let launch_requested = launch_claude;
    app.run(move |app_handle, event| match event {
        tauri::RunEvent::Ready if launch_requested => {
            log::append("app.launch_requested", serde_json::json!({ "source": "desktop-entry" }));
            launch_claude_now(app_handle.clone());
        }
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            // 窗口全部收起时 Tauri 也会请求退出：没点「退出」就不退。
            if code.is_none() && !EXITING.load(Ordering::SeqCst) {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            claude_local::stop_backend();
            claude_local::dictation::server::remove_receipt();
            log::append("app.exit", serde_json::json!({}));
        }
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => show_main_window(app_handle),
        _ => {
            let _ = app_handle;
        }
    });
}

/// 在当前实例里执行「启动 Claude」：桌面图标入口与后来者的唤醒请求共用。
fn launch_claude_now(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let result = commands::claude_local_action("launch".into()).await;
        if result.status == "ok" {
            log::append("claude_local.auto_launch", serde_json::json!({ "status": "ok" }));
        } else {
            log::append(
                "claude_local.auto_launch_failed",
                serde_json::json!({ "message": result.message }),
            );
            show_main_window(&app);
        }
    });
}

fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn install_tray<R: tauri::Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, TRAY_SHOW, "显示 ClaudeMom", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, TRAY_QUIT, "退出（同时停止本地服务）", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let mut builder = TrayIconBuilder::with_id("claude_mom_tray")
        .menu(&menu)
        .tooltip("ClaudeMom")
        .show_menu_on_left_click(!cfg!(windows))
        .on_menu_event(|app, event| match event.id.as_ref() {
            TRAY_SHOW => show_main_window(app),
            TRAY_QUIT => {
                EXITING.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event
                && cfg!(windows)
            {
                show_main_window(tray.app_handle());
            }
        });
    // macOS 菜单栏用单色模板图标（系统按深浅色着色）；其它平台用彩色应用图标。
    #[cfg(target_os = "macos")]
    {
        builder = builder.icon(tauri::include_image!("icons/tray@2x.png")).icon_as_template(true);
    }
    #[cfg(not(target_os = "macos"))]
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}
