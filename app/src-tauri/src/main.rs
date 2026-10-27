#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layer_export;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use tauri::utils::config::BundleType;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use vile_session::{
    KeycodeView, MacroActionDto, MacroUsage, OpDto, SearchHitDto, Session, ViewModel,
};

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
fn open_path(
    state: State<'_, AppState>,
    path: String,
    revision: u64,
    discard: bool,
) -> Result<ViewModel, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        if s.view().is_some_and(|v| v.dirty) && !discard {
            return Err("Save or discard the pending changes before opening a file".into());
        }
        s.open_path(&PathBuf::from(path))
    })
}

// Dialogs block, so these commands are async: Tauri runs them off the main thread.
#[tauri::command]
async fn open_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
    revision: u64,
    discard: bool,
) -> Result<Option<ViewModel>, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        if s.view().is_some_and(|v| v.dirty) && !discard {
            Err("Save or discard changes before opening a file".into())
        } else {
            Ok(())
        }
    })?;
    let picked = app
        .dialog()
        .file()
        .add_filter("Vial layout", &["vil"])
        .blocking_pick_file();
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.open_path(&path)
    })
    .map(Some)
}

#[tauri::command]
fn save(state: State<'_, AppState>, revision: u64) -> Result<ViewModel, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.save()
    })
}

#[tauri::command]
async fn save_as_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
    revision: u64,
) -> Result<Option<ViewModel>, String> {
    with_session(&state, |s| s.check_revision(revision))?;
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
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.save_as(&path)
    })
    .map(Some)
}

#[tauri::command]
async fn export_layers_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
    revision: u64,
    png: Vec<u8>,
) -> Result<Option<String>, String> {
    layer_export::validate_png(&png)?;
    let filename = with_session(&state, |s| {
        s.check_revision(revision)?;
        let view = s.view().ok_or_else(|| "Open a layout first".to_string())?;
        let stem = view
            .path
            .as_deref()
            .and_then(|path| std::path::Path::new(path).file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "layout".into());
        Ok(format!("{stem}-layers.png"))
    })?;
    let picked = app
        .dialog()
        .file()
        .add_filter("PNG image", &["png"])
        .set_file_name(filename)
        .blocking_save_file();
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    with_session(&state, |s| {
        s.check_revision(revision)?;
        let view = s.view().ok_or_else(|| "Open a layout first".to_string())?;
        layer_export::write_png(&path, view.path.as_deref().map(std::path::Path::new), &png)
    })?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
fn apply_op(state: State<'_, AppState>, op: OpDto, revision: u64) -> Result<ViewModel, String> {
    with_session(&state, |s| s.apply_op_at(revision, op))
}

#[tauri::command]
fn undo(state: State<'_, AppState>, revision: u64) -> Result<ViewModel, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.undo()
    })
}

#[tauri::command]
fn redo(state: State<'_, AppState>, revision: u64) -> Result<ViewModel, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.redo()
    })
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
fn set_diff_base(
    state: State<'_, AppState>,
    base: String,
    revision: u64,
) -> Result<ViewModel, String> {
    with_session(&state, |s| {
        s.check_revision(revision)?;
        s.set_diff_base(&base)
    })
}

#[tauri::command]
fn keycode_view(state: State<'_, AppState>, text: String) -> Result<KeycodeView, String> {
    with_session(&state, |s| s.keycode_view(&text))
}
#[tauri::command]
fn compose_keycode(
    state: State<'_, AppState>,
    parts: vile_core::KeyParts,
) -> Result<KeycodeView, String> {
    with_session(&state, |s| s.compose_keycode(parts))
}
#[tauri::command]
fn macro_usage(
    state: State<'_, AppState>,
    actions: Vec<MacroActionDto>,
) -> Result<MacroUsage, String> {
    with_session(&state, |s| s.macro_usage(&actions))
}
#[tauri::command]
fn close_window(
    window: tauri::Window,
    state: State<'_, AppState>,
    revision: u64,
) -> Result<(), String> {
    with_session(&state, |s| s.check_revision(revision))?;
    window.destroy().map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_definition(
    state: State<'_, AppState>,
    revision: u64,
) -> Result<ViewModel, String> {
    let (doc, directory) = with_session(&state, |s| {
        s.check_revision(revision)?;
        Ok((s.document()?.clone(), s.cache_dir().map(PathBuf::from)))
    })?;
    let cache = tauri::async_runtime::spawn_blocking(move || {
        vile_device::read_matching_definition(&doc).map(|cache| (cache, doc))
    })
    .await
    .map_err(|e| e.to_string())??;
    with_session(&state, |s| {
        s.check_revision(revision)?;
        let (cache, doc) = cache;
        let board = cache.board(&doc)?;
        if let Some(directory) = directory {
            cache.save(&directory, &doc)?;
        }
        s.attach_board(board, cache.macro_capacity)
    })
}

fn smoke_enabled() -> bool {
    std::env::var("VILE_SMOKE_TEST").as_deref() == Ok("1")
}

/// The web view calls this once after its first render. In smoke-test mode the
/// app prints what it rendered and exits, so a script can check the whole stack.
#[tauri::command]
fn smoke_report(rendered_keys: u32, layer_count: u32, board_name: String) {
    if smoke_enabled() {
        println!(
            "SMOKE rendered_keys={rendered_keys} layer_count={layer_count} board={board_name}"
        );
        std::process::exit(0);
    }
}

/// Reason shown when this copy of vile cannot install updates on its platform.
const UNSUPPORTED_REASON: &str = "This copy of vile cannot update itself";

/// Reports why this copy cannot install updates, or succeeds when it can.
#[tauri::command]
fn update_support() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err("Development builds do not update".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let bundle = tauri::utils::platform::bundle_type();
    let appimage = std::env::var_os("APPIMAGE").is_some();
    let reason = if cfg!(target_os = "macos") {
        macos_update_reason(&exe)
    } else if cfg!(target_os = "linux") {
        linux_update_reason(bundle, appimage)
    } else if cfg!(target_os = "windows") {
        windows_update_reason(bundle)
    } else {
        Some(UNSUPPORTED_REASON)
    };
    reason.map_or(Ok(()), |reason| Err(reason.into()))
}

// Only a copy inside an app bundle may replace itself. A binary in target/
// would overwrite the build output, and a translocated app is read-only.
fn macos_update_reason(exe: &Path) -> Option<&'static str> {
    let in_bundle = exe.parent().is_some_and(|dir| {
        dir.file_name() == Some(OsStr::new("MacOS"))
            && dir.parent().is_some_and(|contents| {
                contents.file_name() == Some(OsStr::new("Contents"))
                    && contents.parent().and_then(Path::extension) == Some(OsStr::new("app"))
            })
    });
    if !in_bundle {
        Some("This copy of vile is not an app bundle")
    } else if exe.to_string_lossy().contains("/AppTranslocation/") {
        Some("Move vile to the Applications folder to get updates")
    } else {
        None
    }
}

fn linux_update_reason(bundle: Option<BundleType>, appimage: bool) -> Option<&'static str> {
    match bundle {
        Some(BundleType::Deb) => None,
        Some(BundleType::AppImage) if appimage => None,
        _ => Some(UNSUPPORTED_REASON),
    }
}

fn windows_update_reason(bundle: Option<BundleType>) -> Option<&'static str> {
    match bundle {
        Some(BundleType::Nsis) => None,
        _ => Some(UNSUPPORTED_REASON),
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
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState(Mutex::new(session)))
        .setup(|app| {
            let directory = if smoke_enabled() {
                std::env::var_os("VILE_SMOKE_CACHE_DIR")
                    .map(PathBuf::from)
                    .ok_or_else(|| "smoke cache is not configured".to_string())
            } else {
                app.path()
                    .app_data_dir()
                    .map(|dir| dir.join("definitions"))
                    .map_err(|e| e.to_string())
            };
            if let Ok(directory) = directory {
                let state = app.state::<AppState>();
                if let Ok(mut session) = state.0.lock() {
                    session.set_cache_dir(directory);
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("request-close", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_view,
            open_path,
            open_dialog,
            save,
            save_as_dialog,
            export_layers_dialog,
            apply_op,
            undo,
            redo,
            search_keycodes,
            resolve_keycode,
            set_diff_base,
            keycode_view,
            compose_keycode,
            macro_usage,
            close_window,
            refresh_definition,
            smoke_report,
            update_support
        ])
        .build(tauri::generate_context!())
        .expect("error while building vile");
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event
            && !handle.webview_windows().is_empty()
        {
            api.prevent_exit();
            let _ = handle.emit("request-close", ());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_bundle_copy_can_update() {
        let exe = Path::new("/Applications/vile.app/Contents/MacOS/vile-app");
        assert_eq!(macos_update_reason(exe), None);
    }

    #[test]
    fn translocated_copy_asks_for_the_applications_folder() {
        let exe = Path::new(
            "/private/var/folders/x1/T/AppTranslocation/3F2A/d/vile.app/Contents/MacOS/vile-app",
        );
        assert_eq!(
            macos_update_reason(exe),
            Some("Move vile to the Applications folder to get updates")
        );
    }

    #[test]
    fn bare_build_binary_is_not_a_bundle() {
        let exe = Path::new("/Users/me/git/vile/target/release/vile-app");
        assert_eq!(
            macos_update_reason(exe),
            Some("This copy of vile is not an app bundle")
        );
    }

    #[test]
    fn deb_and_appimage_copies_can_update() {
        assert_eq!(linux_update_reason(Some(BundleType::Deb), false), None);
        assert_eq!(linux_update_reason(Some(BundleType::AppImage), true), None);
    }

    #[test]
    fn appimage_without_its_env_and_unbundled_copies_cannot_update() {
        assert_eq!(
            linux_update_reason(Some(BundleType::AppImage), false),
            Some(UNSUPPORTED_REASON)
        );
        assert_eq!(linux_update_reason(None, false), Some(UNSUPPORTED_REASON));
    }

    #[test]
    fn only_the_nsis_installer_copy_can_update_on_windows() {
        assert_eq!(windows_update_reason(Some(BundleType::Nsis)), None);
        assert_eq!(
            windows_update_reason(Some(BundleType::Msi)),
            Some(UNSUPPORTED_REASON)
        );
    }
}
