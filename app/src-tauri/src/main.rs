#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use vile_session::{OpDto, SearchHitDto, Session, ViewModel};

/// The single editing session, shared by all commands.
struct AppState(Mutex<Session>);

fn with_session<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&mut Session) -> Result<T, String>,
) -> Result<T, String> {
    let mut session = state
        .0
        .lock()
        .map_err(|_| "internal error: the session lock is poisoned".to_string())?;
    f(&mut session)
}

#[tauri::command]
fn get_view(state: State<'_, AppState>) -> Result<Option<ViewModel>, String> {
    with_session(&state, |s| Ok(s.view()))
}

#[tauri::command]
fn open_path(state: State<'_, AppState>, path: String) -> Result<ViewModel, String> {
    with_session(&state, |s| s.open_path(&PathBuf::from(path)))
}

// Dialogs block, so these commands are async: Tauri runs them off the main thread.
#[tauri::command]
async fn open_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ViewModel>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Vial layout", &["vil"])
        .blocking_pick_file();
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    with_session(&state, |s| s.open_path(&path)).map(Some)
}

#[tauri::command]
fn save(state: State<'_, AppState>) -> Result<ViewModel, String> {
    with_session(&state, |s| s.save())
}

#[tauri::command]
async fn save_as_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ViewModel>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Vial layout", &["vil"])
        .set_file_name("layout.vil")
        .blocking_save_file();
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    with_session(&state, |s| s.save_as(&path)).map(Some)
}

#[tauri::command]
fn apply_op(state: State<'_, AppState>, op: OpDto) -> Result<ViewModel, String> {
    with_session(&state, |s| s.apply_op(op))
}

#[tauri::command]
fn undo(state: State<'_, AppState>) -> Result<ViewModel, String> {
    with_session(&state, |s| s.undo())
}

#[tauri::command]
fn redo(state: State<'_, AppState>) -> Result<ViewModel, String> {
    with_session(&state, |s| s.redo())
}

#[tauri::command]
fn search_keycodes(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchHitDto>, String> {
    with_session(&state, |s| Ok(s.search_keycodes(&query, limit)))
}

#[tauri::command]
fn resolve_keycode(state: State<'_, AppState>, text: String) -> Result<String, String> {
    with_session(&state, |s| s.resolve_keycode(&text))
}

#[tauri::command]
fn set_diff_base(state: State<'_, AppState>, base: String) -> Result<ViewModel, String> {
    with_session(&state, |s| s.set_diff_base(&base))
}

fn smoke_enabled() -> bool {
    std::env::var("VILE_SMOKE_TEST").as_deref() == Ok("1")
}

/// The web view calls this once after its first render. In smoke-test mode the
/// app prints what it rendered and exits, so a script can check the whole stack.
#[tauri::command]
fn smoke_report(rendered_keys: u32, layer_count: u32) {
    if smoke_enabled() {
        println!("SMOKE rendered_keys={rendered_keys} layer_count={layer_count}");
        std::process::exit(0);
    }
}

fn main() {
    if smoke_enabled() {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(30));
            println!("SMOKE TIMEOUT");
            std::process::exit(2);
        });
    }
    let mut session = Session::new();
    if let Some(arg) = std::env::args().nth(1)
        && let Err(e) = session.open_path(&PathBuf::from(&arg))
    {
        eprintln!("vile: cannot open {arg}: {e}");
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(session)))
        .invoke_handler(tauri::generate_handler![
            get_view,
            open_path,
            open_dialog,
            save,
            save_as_dialog,
            apply_op,
            undo,
            redo,
            search_keycodes,
            resolve_keycode,
            set_diff_base,
            smoke_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running vile");
}
