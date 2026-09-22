#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod config;
mod ipc;
mod markdown;
mod tab;

use config::Config;
use ipc::UserEvent;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tab::DocumentTab;
use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::{WebContext, WebViewBuilder};

static NEXT_TAB_ID: AtomicUsize = AtomicUsize::new(1);

const ICON_RGBA: &[u8] = include_bytes!("../assets/icon_32.rgba");

fn load_window_icon() -> Option<tao::window::Icon> {
    tao::window::Icon::from_rgba(ICON_RGBA.to_vec(), 32, 32).ok()
}

fn get_next_tab_id() -> usize {
    NEXT_TAB_ID.fetch_add(1, Ordering::SeqCst)
}

fn get_local_app_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into())))
        .join("MarkdownReader")
}

fn log_startup_error(err_msg: &str) {
    let local_dir = get_local_app_dir();
    let _ = std::fs::create_dir_all(&local_dir);
    let log_path = local_dir.join("startup.log");
    let now = std::time::SystemTime::now();
    let entry = format!("[{:?}] Startup Error: {}\n", now, err_msg);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, entry.as_bytes()));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let initial_file_path = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if path.exists() {
            Some(path.canonicalize().unwrap_or(path))
        } else {
            Some(path)
        }
    } else {
        None
    };

    // Single-instance check: try to send file path to already running instance
    if ipc::try_send_to_existing_instance(initial_file_path.as_deref()) {
        // Successfully sent to existing instance, exit early!
        return Ok(());
    }

    let mut config = Config::load();
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Start background IPC server for single-instance command line forwarding
    ipc::start_ipc_server(proxy.clone());

    let mut window_builder = WindowBuilder::new()
        .with_title("MarkdownReader")
        .with_inner_size(LogicalSize::new(config.width as f64, config.height as f64));

    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    if config.maximized {
        window_builder = window_builder.with_maximized(true);
    }

    let window = window_builder.build(&event_loop)?;

    // Restore multi-monitor valid position
    if !config.maximized && config.is_position_valid(&window) {
        if let (Some(x), Some(y)) = (config.x, config.y) {
            window.set_outer_position(PhysicalPosition::new(x, y));
        }
    }

    let shell_html = markdown::build_shell_html(&config);

    let local_data_dir = get_local_app_dir();
    let webview_data_dir = local_data_dir.join("WebView2");
    if let Err(e) = std::fs::create_dir_all(&webview_data_dir) {
        log_startup_error(&format!("Failed to create WebView2 data directory {:?}: {}", webview_data_dir, e));
    }

    let mut web_context = WebContext::new(Some(webview_data_dir.clone()));

    let proxy_for_ipc = proxy.clone();
    let webview_builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_custom_protocol("mdasset".into(), move |_id, req| {
            assets::handle_asset_request(req)
        })
        .with_navigation_handler(move |uri| {
            // Anchor links within the document
            if uri.starts_with('#') || uri.starts_with("about:blank#") {
                return true;
            }
            // Allow custom asset protocol
            if uri.starts_with("http://mdasset.localhost") || uri.starts_with("https://mdasset.localhost") {
                return true;
            }
            // External links (http, https, mailto) open in default browser
            if uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("mailto:") {
                let _ = open::that(&uri);
                return false;
            }
            true
        })
        .with_ipc_handler(move |req| {
            let body = req.body();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(action) = parsed.get("action").and_then(|a| a.as_str()) {
                    match action {
                        "open_file" => {
                            let proxy = proxy_for_ipc.clone();
                            std::thread::spawn(move || {
                                if let Some(file) = rfd::FileDialog::new()
                                    .add_filter("Markdown", &["md", "markdown", "txt"])
                                    .pick_file()
                                {
                                    let _ = proxy.send_event(UserEvent::OpenFile(file));
                                }
                            });
                        }
                        "tab_changed" => {
                            if let Some(title) = parsed.get("title").and_then(|t| t.as_str()) {
                                let _ = proxy_for_ipc.send_event(UserEvent::UpdateTitle(title.to_string()));
                            }
                        }
                        "all_tabs_closed" => {
                            let _ = proxy_for_ipc.send_event(UserEvent::OpenWelcome);
                        }
                        _ => {}
                    }
                }
            }
        })
        .with_html(&shell_html);

    let webview = match webview_builder.build(&window) {
        Ok(wv) => wv,
        Err(err) => {
            let err_desc = format!(
                "Failed to initialize WebView2 runtime:\n{}\n\nUser Data Directory:\n{}\n\nPlease verify that Microsoft Edge WebView2 Runtime is installed.",
                err,
                webview_data_dir.display()
            );
            log_startup_error(&err_desc);
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("MarkdownReader - Startup Error")
                .set_description(&err_desc)
                .show();
            return Err(err.into());
        }
    };

    // Load initial tab
    let initial_tab = if let Some(path) = initial_file_path {
        DocumentTab::from_file(get_next_tab_id(), &path).unwrap_or_else(|_| DocumentTab::new_welcome(get_next_tab_id()))
    } else {
        DocumentTab::new_welcome(get_next_tab_id())
    };

    if let Ok(json_str) = serde_json::to_string(&initial_tab) {
        let script = format!("window.addOrActivateTab({}, true);", json_str);
        let _ = webview.evaluate_script(&script);
        window.set_title(&format!("{} - MarkdownReader", initial_tab.title));
    }

    let _ = webview.focus();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::OpenFile(path)) => {
                let tab = match DocumentTab::from_file(get_next_tab_id(), &path) {
                    Ok(t) => t,
                    Err(e) => {
                        let id = get_next_tab_id();
                        let error_body = format!(
                            "<h1>Error reading file</h1><p>Could not open <code>{}</code>: {}</p>",
                            path.display(),
                            e
                        );
                        DocumentTab {
                            id,
                            path: Some(path.clone()),
                            title: path.file_name().and_then(|n| n.to_str()).unwrap_or("Error").to_string(),
                            html_body: error_body,
                            scroll_y: 0.0,
                        }
                    }
                };

                if let Ok(json_str) = serde_json::to_string(&tab) {
                    let script = format!("window.addOrActivateTab({}, true);", json_str);
                    let _ = webview.evaluate_script(&script);
                    window.set_title(&format!("{} - MarkdownReader", tab.title));
                }

                window.set_minimized(false);
                window.set_focus();
            }
            Event::UserEvent(UserEvent::UpdateTitle(title)) => {
                window.set_title(&format!("{} - MarkdownReader", title));
            }
            Event::UserEvent(UserEvent::OpenWelcome) => {
                let id = get_next_tab_id();
                let welcome = DocumentTab::new_welcome(id);
                if let Ok(json_str) = serde_json::to_string(&welcome) {
                    let script = format!("window.addOrActivateTab({}, true);", json_str);
                    let _ = webview.evaluate_script(&script);
                    window.set_title("Welcome - MarkdownReader");
                }
            }
            Event::UserEvent(UserEvent::ActivateWindow) => {
                window.set_minimized(false);
                window.set_focus();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                config.update_from_window(&window);
                let _ = config.save();
                *control_flow = ControlFlow::Exit;
            }
            Event::LoopDestroyed => {
                config.update_from_window(&window);
                let _ = config.save();
            }
            _ => (),
        }
    });
}
