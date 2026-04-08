

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
struct TogglePipEvent {
    toggle: bool,
}

#[derive(Clone, Serialize)]
struct ScreenshotCreatedEvent {
    filename: String,
}

#[derive(Clone, Serialize)]
struct ProfileLogEvent {
    #[serde(rename = "profileId")]
    profile_id: String,
    #[serde(rename = "accountId")]
    account_id: String,
}

#[derive(Clone, Serialize)]
struct RaidLogEvent {
    #[serde(rename = "profileId")]
    profile_id: String,
    status: String,
    #[serde(rename = "raidMode")]
    raid_mode: String,
    ip: String,
    port: String,
    location: String,
    sid: String,
    #[serde(rename = "gameMode")]
    game_mode: String,
    #[serde(rename = "shortId")]
    short_id: String,
    #[serde(rename = "realTime", skip_serializing_if = "Option::is_none")]
    real_time: Option<String>,
}

struct AppState {
    screenshot_path: Mutex<String>,
    tarkov_game_path: Mutex<String>,
}

// Start global keyboard listener
fn start_global_keyboard_listener(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let callback = move |event: rdev::Event| {
            if let rdev::EventType::KeyPress(key) = event.event_type {
                // Only handle M key
                if matches!(key, rdev::Key::KeyM) {
                    // Send toggle PiP event to frontend
                    if let Err(e) = app_handle.emit("toggle-pip", &TogglePipEvent { toggle: true })
                    {
                        eprintln!("Failed to emit toggle-pip event: {}", e);
                    }
                }
            }
        };

        // Start listening, this will block the current thread
        if let Err(error) = rdev::listen(callback) {
            eprintln!("Error while listening to keyboard events: {:?}", error);
        }
    });
}



#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file {}: {}", path, e))
}

#[tauri::command]
fn read_directory(path: String) -> Result<Vec<String>, String> {
    let entries =
        fs::read_dir(&path).map_err(|e| format!("Failed to read directory {}: {}", path, e))?;

    let mut files = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
    }

    Ok(files)
}

#[tauri::command]
fn path_exists(path: String) -> bool {
    PathBuf::from(path).exists()
}

#[tauri::command]
fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_screenshot_path(state: tauri::State<AppState>, path: String) -> Result<String, String> {
    let mut screenshot_path = state.screenshot_path.lock().unwrap();
    *screenshot_path = path.clone();
    Ok(format!("Screenshot path set to: {}", path))
}

#[tauri::command]
fn get_screenshot_path(state: tauri::State<AppState>) -> String {
    let screenshot_path = state.screenshot_path.lock().unwrap();
    screenshot_path.clone()
}

#[tauri::command]
fn set_tarkov_game_path(state: tauri::State<AppState>, path: String) -> Result<String, String> {
    let mut game_path = state.tarkov_game_path.lock().unwrap();
    *game_path = path.clone();
    Ok(format!("Tarkov game path set: {}", path))
}

#[tauri::command]
fn get_tarkov_game_path(state: tauri::State<AppState>) -> String {
    let game_path = state.tarkov_game_path.lock().unwrap();
    game_path.clone()
}

#[tauri::command]
async fn open_pip_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    if let Some(_) = app.get_webview_window("pip") {
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        "pip",
        tauri::WebviewUrl::App("index.html#/pip".into())
    )
    .title("Map PiP")
    .inner_size(600.0, 450.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn close_pip_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    if let Some(window) = app.get_webview_window("pip") {
        window.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn resolve_application_log_path(game_root: &str) -> Option<PathBuf> {
    let root = Path::new(game_root);
    if !root.is_dir() {
        return None;
    }
    let logs_dir = root.join("Logs");
    if !logs_dir.is_dir() {
        return None;
    }
    let mut latest_log_dir: Option<(std::time::SystemTime, PathBuf)> = None;
    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // log_YYYY.MM.DD_HH-mm-ss_*
                if name.starts_with("log_") && name.len() > 4 {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if latest_log_dir.as_ref().map(|(t, _)| modified > *t).unwrap_or(true) {
                                latest_log_dir = Some((modified, path));
                            }
                        }
                    }
                }
            }
        }
    }
    let log_dir = latest_log_dir.map(|(_, p)| p)?;
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.to_lowercase().contains("application") {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn split_log_lines(content: &str) -> Vec<String> {
    let mut logs = Vec::new();
    let mut buffer = String::new();
    for line in content.lines() {
        let is_new = line.len() >= 19
            && line.as_bytes().get(4) == Some(&b'-')
            && line.as_bytes().get(7) == Some(&b'-')
            && line.as_bytes().get(10) == Some(&b' ');
        if is_new {
            if !buffer.is_empty() {
                logs.push(buffer.trim().to_string());
            }
            buffer = line.to_string();
        } else {
            buffer.push('\n');
            buffer.push_str(line);
        }
    }
    if !buffer.is_empty() {
        logs.push(buffer.trim().to_string());
    }
    logs
}

fn profile_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)SelectProfile ProfileId:(.+?) AccountId:(.+)").unwrap())
}

fn raid_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)'Profileid: (.+?), Status: (.+?), RaidMode: (.+?), Ip: (.+?), Port: (.+?), Location: (.+?), Sid: (.+?), GameMode: (.+?), shortId: (.+?)'").unwrap())
}

fn sid_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(.+?)-(.+?)_(.+)").unwrap())
}

fn parse_profile_line(text: &str) -> Option<ProfileLogEvent> {
    let cap = profile_regex().captures(text)?;
    Some(ProfileLogEvent {
        profile_id: cap.get(1)?.as_str().to_string(),
        account_id: cap.get(2)?.as_str().trim().to_string(),
    })
}

fn parse_raid_line(text: &str) -> Option<RaidLogEvent> {
    let cap = raid_regex().captures(text)?;
    let sid = cap.get(7)?.as_str().to_string();
    let real_time = sid_regex()
        .captures(&sid)
        .and_then(|c| Some(c.get(3)?.as_str().to_string()));
    Some(RaidLogEvent {
        profile_id: cap.get(1)?.as_str().to_string(),
        status: cap.get(2)?.as_str().to_string(),
        raid_mode: cap.get(3)?.as_str().to_string(),
        ip: cap.get(4)?.as_str().to_string(),
        port: cap.get(5)?.as_str().to_string(),
        location: cap.get(6)?.as_str().to_string(),
        sid,
        game_mode: cap.get(8)?.as_str().to_string(),
        short_id: cap.get(9)?.as_str().to_string(),
        real_time,
    })
}

// ── Monitor control (Windows only) ──────────────────────────────────────────

#[cfg(windows)]
mod monitor_ctrl {
    use serde::Serialize;
    use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        CreateDCW, DeleteDC, EnumDisplayMonitors, GetMonitorInfoW, HDC,
        HMONITOR, MONITORINFO, MONITORINFOEXW,
    };
    use windows::Win32::UI::ColorSystem::SetDeviceGammaRamp;

    #[derive(Clone, Serialize)]
    pub struct MonitorItem {
        pub index: usize,
        pub name: String,
        pub device: String,
    }

    pub fn enumerate() -> Vec<MonitorItem> {
        let mut list: Vec<MonitorItem> = Vec::new();
        unsafe {
            let success = EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_proc),
                LPARAM(&mut list as *mut _ as isize),
            );
            if !success.as_bool() {
                eprintln!("枚举显示器失败");
            }
        }
        list
    }

    unsafe extern "system" fn enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let list = &mut *(lparam.0 as *mut Vec<MonitorItem>);
        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(
            hmonitor,
            &mut info.monitorInfo as *mut _ as *mut MONITORINFO,
        )
        .as_bool()
        {
            let device = String::from_utf16_lossy(&info.szDevice)
                .trim_end_matches('\0')
                .to_string();
            let index = list.len();
            list.push(MonitorItem {
                index,
                name: format!("屏幕 {}", index + 1),
                device,
            });
        }
        BOOL(1)
    }

    fn build_ramp(gamma: f64, brightness: f64, contrast: f64, channel_scale: f64) -> [u16; 256] {
        let mut ramp = [0u16; 256];
        for i in 0usize..256 {
            let n = i as f64 / 255.0;
            // gamma correction
            let g = n.powf(1.0 / gamma.max(0.01));
            // brightness (100 = neutral)
            let b = g * (brightness / 100.0);
            // contrast (100 = neutral): stretch around midpoint
            let c = (b - 0.5) * (contrast / 100.0) + 0.5;
            // per-channel scale (128 = neutral → 1.0)
            let v = c * channel_scale;
            ramp[i] = (v.clamp(0.0, 1.0) * 65535.0) as u16;
        }
        ramp
    }

    pub fn set_gamma_ramp(
        device: &str,
        gamma: f64,
        brightness: f64,
        contrast: f64,
        red: f64,
        green: f64,
        blue: f64,
    ) -> bool {
        let r = build_ramp(gamma, brightness, contrast, red / 128.0);
        let g = build_ramp(gamma, brightness, contrast, green / 128.0);
        let b = build_ramp(gamma, brightness, contrast, blue / 128.0);

        let mut ramp_data = [0u16; 768];
        ramp_data[0..256].copy_from_slice(&r);
        ramp_data[256..512].copy_from_slice(&g);
        ramp_data[512..768].copy_from_slice(&b);

        unsafe {
            let wide: Vec<u16> = device.encode_utf16().chain(std::iter::once(0)).collect();
            let hdc = CreateDCW(
                windows::core::PCWSTR(wide.as_ptr()),
                windows::core::PCWSTR::null(),
                windows::core::PCWSTR::null(),
                None,
            );
            if hdc.is_invalid() {
                return false;
            }
            let ok = SetDeviceGammaRamp(hdc, ramp_data.as_ptr() as *const _).as_bool();
            let _ = DeleteDC(hdc);
            ok
        }
    }

    pub fn reset_gamma_ramp(device: &str) -> bool {
        // Linear ramp: ramp[i] = i * 257  (maps 0..255 → 0..65535)
        let linear: [u16; 256] = std::array::from_fn(|i| (i as u16) * 257);
        let mut ramp_data = [0u16; 768];
        ramp_data[0..256].copy_from_slice(&linear);
        ramp_data[256..512].copy_from_slice(&linear);
        ramp_data[512..768].copy_from_slice(&linear);

        unsafe {
            let wide: Vec<u16> = device.encode_utf16().chain(std::iter::once(0)).collect();
            let hdc = CreateDCW(
                windows::core::PCWSTR(wide.as_ptr()),
                windows::core::PCWSTR::null(),
                windows::core::PCWSTR::null(),
                None,
            );
            if hdc.is_invalid() {
                return false;
            }
            let ok = SetDeviceGammaRamp(hdc, ramp_data.as_ptr() as *const _).as_bool();
            let _ = DeleteDC(hdc);
            ok
        }
    }
}

#[tauri::command]
fn get_monitors() -> Vec<serde_json::Value> {
    #[cfg(windows)]
    {
        monitor_ctrl::enumerate()
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "index": m.index,
                    "name": m.name,
                    "device": m.device,
                })
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![serde_json::json!({"index": 0, "name": "屏幕 1", "device": ""})]
    }
}

#[tauri::command]
fn set_monitor_params(
    device: String,
    gamma: f64,
    brightness: f64,
    contrast: f64,
    red: f64,
    green: f64,
    blue: f64,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        if monitor_ctrl::set_gamma_ramp(&device, gamma, brightness, contrast, red, green, blue) {
            Ok(())
        } else {
            Err(format!("SetDeviceGammaRamp failed for {}", device))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (device, gamma, brightness, contrast, red, green, blue);
        Ok(())
    }
}

#[tauri::command]
fn reset_monitor_params(device: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        if monitor_ctrl::reset_gamma_ramp(&device) {
            Ok(())
        } else {
            Err(format!("Reset gamma ramp failed for {}", device))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = device;
        Ok(())
    }
}

fn cleanup_screenshot_pngs(screenshot_path: &str) {
    if screenshot_path.is_empty() {
        return;
    }

    let path = Path::new(screenshot_path);
    if !path.exists() || !path.is_dir() {
        return;
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if let Some(extension) = file_path.extension() {
                            if extension.to_str().unwrap_or("").to_lowercase() == "png" {
                                let _ = fs::remove_file(&file_path);
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::{
        menu::{MenuBuilder, MenuItemBuilder},
        tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    };

    // Initialize app state
    let app_state = AppState {
        screenshot_path: Mutex::new(String::new()),
        tarkov_game_path: Mutex::new(String::new()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.show();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Create system tray menu
            let show_item = MenuItemBuilder::new("Show Window").id("show").build(app)?;
            let hide_item = MenuItemBuilder::new("Hide Window").id("hide").build(app)?;
            let quit_item = MenuItemBuilder::new("Quit").id("quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create system tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        // Cleanup PNG files before exiting
                        if let Some(state) = app.try_state::<AppState>() {
                            let screenshot_path = state.screenshot_path.lock().unwrap();
                            cleanup_screenshot_pngs(&screenshot_path);
                        }

                        // Explicitly close the window before exiting to avoid "Failed to unregister class" error
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.destroy();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide window on close instead of exit and block refresh shortcuts
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });

                // Block refresh shortcuts (F5, Ctrl+F5, Ctrl+R, Shift+Ctrl+R)
                let _ = window.eval(
                    r#"
                    window.addEventListener('keydown', function(e) {
                        if (e.key === 'F5') {
                            e.preventDefault();
                            e.stopPropagation();
                            return false;
                        }

                        if (e.ctrlKey && e.key === 'F5') {
                            e.preventDefault();
                            e.stopPropagation();
                            return false;
                        }

                        if (e.shiftKey && e.ctrlKey && e.key === 'R') {
                            e.preventDefault();
                            e.stopPropagation();
                            return false;
                        }
                        
                        if (e.ctrlKey && !e.shiftKey && e.key === 'r') {
                            e.preventDefault();
                            e.stopPropagation();
                            return false;
                        }
                    }, true);
                    "#,
                );
            }

            // Start global keyboard listener
            start_global_keyboard_listener(app.handle().clone());

            // Start screenshot directory watcher in a background thread
            let app_handle_screenshot = app.handle().clone();
            thread::spawn(move || {
                use std::collections::HashSet;

                let mut known_files: HashSet<String> = HashSet::new();
                loop {
                    let path = {
                        let state = app_handle_screenshot.state::<AppState>();
                        let p = state.screenshot_path.lock().unwrap();
                        p.clone()
                    };

                    if !path.is_empty() {
                        if let Ok(entries) = fs::read_dir(&path) {
                            let mut current_files: HashSet<String> = HashSet::new();
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    if let Some(name) = entry.file_name().to_str() {
                                        let name_str = name.to_string();
                                        if !known_files.contains(&name_str) {
                                            let _ = app_handle_screenshot.emit(
                                                "screenshot-created",
                                                &ScreenshotCreatedEvent {
                                                    filename: name_str.clone(),
                                                },
                                            );
                                        }
                                        current_files.insert(name_str);
                                    }
                                }
                            }
                            known_files = current_files;
                        }
                    }

                    thread::sleep(Duration::from_millis(1000));
                }
            });

            // Start game log watcher in a separate background thread
            let app_handle_log = app.handle().clone();
            thread::spawn(move || {
                let mut last_line_count: u64 = 0;
                let mut last_modified: Option<std::time::SystemTime> = None;
                let mut initialized = false;
                loop {
                    let game_root = {
                        let state = app_handle_log.state::<AppState>();
                        let p = state.tarkov_game_path.lock().unwrap();
                        p.clone()
                    };

                    if !game_root.is_empty() {
                        if let Some(log_path) = resolve_application_log_path(&game_root) {
                            if let Ok(meta) = fs::metadata(&log_path) {
                                let modified = meta.modified().ok();
                                let need_read = match (last_modified, modified) {
                                    (None, Some(_)) => true,
                                    (Some(a), Some(b)) => b > a,
                                    _ => false,
                                };
                                if need_read {
                                    last_modified = modified;
                                    if let Ok(content) = fs::read_to_string(&log_path) {
                                        let lines = split_log_lines(&content);
                                        let n = lines.len() as u64;
                                        if n > last_line_count {
                                            let start = last_line_count as usize;
                                            if !initialized {
                                                initialized = true;
                                                last_line_count = n;
                                            } else {
                                                last_line_count = n;
                                                for line in lines.iter().skip(start) {
                                                    if let Some(ev) = parse_profile_line(line) {
                                                        let _ = app_handle_log.emit(
                                                            "profile-log",
                                                            &ProfileLogEvent {
                                                                profile_id: ev.profile_id,
                                                                account_id: ev.account_id,
                                                            },
                                                        );
                                                    }
                                                    if let Some(ev) = parse_raid_line(line) {
                                                        let _ = app_handle_log.emit("raid-log", &ev);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        last_line_count = 0;
                        last_modified = None;
                        initialized = false;
                    }

                    thread::sleep(Duration::from_millis(2000));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read_text_file,
            read_directory,
            path_exists,
            minimize_window,
            set_screenshot_path,
            get_screenshot_path,
            set_tarkov_game_path,
            get_tarkov_game_path,
            open_pip_window,
            close_pip_window,
            get_monitors,
            set_monitor_params,
            reset_monitor_params,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
