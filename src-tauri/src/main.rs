#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod timer;

use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

use timer::{Phase, Timer};

const WINDOW_LABEL: &str = "main";
const EVENT_TICK: &str = "countdown://tick";
const EVENT_FINISHED: &str = "countdown://finished";
const EVENT_SYNC: &str = "countdown://sync";

struct AppState {
    timer: Mutex<Timer>,
}

#[derive(Serialize, Clone)]
struct Snapshot {
    phase: &'static str,
    remaining_ms: u64,
}

#[derive(Serialize, Clone)]
struct TickPayload {
    remaining_ms: u64,
}

fn lock(state: &AppState) -> MutexGuard<'_, Timer> {
    state.timer.lock().unwrap_or_else(|err| err.into_inner())
}

/// 把主窗口叫回屏幕最前：显示 → 取消最小化 → 取焦点 → 通知前端刷新
fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = app.emit(EVENT_SYNC, ());
    }
}

#[tauri::command]
fn start_countdown(seconds: u64, state: State<'_, AppState>) -> Result<(), String> {
    if seconds == 0 {
        return Err("倒计时时长必须大于 0".to_string());
    }
    lock(&state).start(seconds);
    Ok(())
}

#[tauri::command]
fn pause_countdown(state: State<'_, AppState>) -> Result<(), String> {
    if lock(&state).pause() {
        Ok(())
    } else {
        Err("当前状态不能暂停".to_string())
    }
}

#[tauri::command]
fn resume_countdown(state: State<'_, AppState>) -> Result<(), String> {
    if lock(&state).resume() {
        Ok(())
    } else {
        Err("当前状态不能继续".to_string())
    }
}

#[tauri::command]
fn cancel_countdown(state: State<'_, AppState>) {
    lock(&state).cancel();
}

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> Snapshot {
    let timer = lock(&state);
    Snapshot {
        phase: timer.phase().as_str(),
        remaining_ms: timer.remaining_ms(),
    }
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

/// 100ms 轮询：整秒变化时推送 tick，归零时把窗口叫回来
fn poll_loop(app: AppHandle) {
    let mut last_second: Option<u64> = None;
    loop {
        std::thread::sleep(Duration::from_millis(100));

        let state = app.state::<AppState>();
        let (phase, remaining_ms) = {
            let timer = lock(&state);
            (timer.phase(), timer.remaining_ms())
        };

        if phase != Phase::Running {
            last_second = None;
            continue;
        }

        if remaining_ms == 0 {
            {
                let mut timer = lock(&state);
                timer.finish();
            }
            last_second = Some(0);
            if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            let _ = app.emit(EVENT_TICK, TickPayload { remaining_ms: 0 });
            let _ = app.emit(EVENT_FINISHED, ());
            continue;
        }

        let second = remaining_ms.div_ceil(1000);
        if last_second != Some(second) {
            last_second = Some(second);
            let _ = app.emit(EVENT_TICK, TickPayload { remaining_ms });
        }
    }
}

/// acrylic 会铺满整个窗口矩形，用 Win11 的 DWM 圆角把它裁成圆角
fn round_window_corners(window: &tauri::WebviewWindow) {
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            attr: u32,
            value: *const std::ffi::c_void,
            size: u32,
        ) -> i32;
    }

    if let Ok(hwnd) = window.hwnd() {
        let preference: u32 = DWMWCP_ROUND;
        unsafe {
            DwmSetWindowAttribute(
                hwnd.0 as isize,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("倒计时")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        // 单实例：重复启动时只唤醒已有窗口，不再开第二个进程
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .manage(AppState {
            timer: Mutex::new(Timer::new()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            // 让 Windows 的 acrylic 底色跟随深色主题，避免浅色系统下玻璃泛白
            if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                let _ = window.set_theme(Some(tauri::Theme::Dark));
                round_window_corners(&window);
            }
            build_tray(&handle)?;
            std::thread::spawn(move || poll_loop(handle));
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关闭按钮不退出进程，只隐藏到托盘
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_countdown,
            pause_countdown,
            resume_countdown,
            cancel_countdown,
            get_state,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("启动倒计时应用失败");
}
