use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

/// How long a copied secret stays on the clipboard before we try to clear it.
const CLIPBOARD_CLEAR_SECONDS: u64 = 45;

/// Monotonic id of the most recent clipboard copy, used to let only the newest
/// scheduled clear act on the clipboard.
static CLIPBOARD_CLEAR_GENERATION: AtomicU64 = AtomicU64::new(0);

fn next_clipboard_clear_generation() -> u64 {
    CLIPBOARD_CLEAR_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}

fn is_current_clipboard_generation(generation: u64) -> bool {
    CLIPBOARD_CLEAR_GENERATION.load(Ordering::SeqCst) == generation
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SystemStatus {
    has_gpg: bool,
    has_pass: bool,
    has_pass_store: bool,
    home_dir: String,
}

/// Resolve `~/.password-store`, returning a displayable error when the home
/// directory can't be determined instead of panicking.
fn password_store_dir() -> Result<PathBuf, String> {
    let home_dir = env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    Ok(home_dir.join(".password-store"))
}

/// Return `pass <entry>`'s first line -- the password itself, not the rest of
/// the (possibly multiline) entry.
fn first_line(output: &str) -> &str {
    output.lines().next().unwrap_or("")
}

/// Schedule the clipboard to be cleared after `CLIPBOARD_CLEAR_SECONDS`, but
/// only when it still holds the value we wrote (mirrors `pass -c` semantics).
///
/// Every copy takes a new generation number and supersedes the previous timer:
/// re-copying within the window restarts the 45s clock instead of letting the
/// older timer clear the secret the newer copy just wrote.
fn schedule_clipboard_clear(app: &tauri::AppHandle, expected: String) {
    let app = app.clone();
    let generation = next_clipboard_clear_generation();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(CLIPBOARD_CLEAR_SECONDS));

        // A newer copy owns the clear now; leaving the clipboard alone keeps
        // its full 45s window intact.
        if !is_current_clipboard_generation(generation) {
            return;
        }

        if let Ok(current) = app.clipboard().read_text() {
            if current == expected {
                let _ = app.clipboard().clear();
            }
        }
    });
}

#[tauri::command]
async fn initialize() -> Result<SystemStatus, String> {
    let password_store = password_store_dir()?;
    let home_dir = env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;

    Ok(SystemStatus {
        has_gpg: Command::new("gpg").arg("--version").output().is_ok(),
        has_pass: Command::new("pass").arg("--version").output().is_ok(),
        has_pass_store: password_store.exists() && password_store.is_dir(),
        home_dir: home_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn load_password_store() -> Result<Vec<String>, String> {
    let password_store = password_store_dir()?;
    search_for_gpg_files(&password_store)
}

/// Recursively collect `.gpg` entries under `path`. Hidden files/folders are
/// ignored, non-`.gpg` files are skipped, and unreadable subdirectories are
/// skipped gracefully rather than panicking.
fn search_for_gpg_files(path: &Path) -> Result<Vec<String>, String> {
    let mut result = Vec::new();

    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?;

    for entry in entries {
        // Skip anything we can't stat/read.
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();

        // ignore hidden files and folders
        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if file_name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            // A directory we can't read shouldn't take the whole list down.
            if let Ok(sub_entries) = search_for_gpg_files(&path) {
                result.extend(sub_entries);
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("gpg") {
            result.push(path.display().to_string());
        }
    }

    Ok(result)
}

#[tauri::command]
async fn copy_encrypted_password_to_clipboard(
    relative_path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let output = Command::new("pass")
        .arg(&relative_path)
        .output()
        .map_err(|e| format!("Failed to run pass: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error decrypting password: {}", stderr.trim()));
    }

    // Copy only the password (first line), not the rest of the entry.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let password = first_line(&stdout).to_string();

    app.clipboard()
        .write_text(password.clone())
        .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

    schedule_clipboard_clear(&app, password.clone());

    Ok("Copied to clipboard!".to_string())
}

#[tauri::command]
fn copy_text_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text.clone())
        .map_err(|e| format!("Failed to write to clipboard: {e}"))?;

    schedule_clipboard_clear(&app, text);

    Ok(())
}

#[tauri::command]
async fn open_dialog(app: tauri::AppHandle) -> Result<String, String> {
    let password_store = password_store_dir()?;

    let file_path = app
        .dialog()
        .file()
        .add_filter("Folder", &["folder"])
        .set_directory(password_store)
        .set_title("Select a folder to save your new password")
        .blocking_pick_folder()
        .ok_or_else(|| "No folder selected".to_string())?;

    Ok(file_path.to_string())
}

#[tauri::command]
fn generate_new_pass(
    store_relative_path: String,
    length: u32,
    no_symbols: bool,
) -> Result<String, String> {
    // Never silently overwrite an existing entry.
    let password_store = password_store_dir()?;
    let target = password_store.join(format!("{store_relative_path}.gpg"));
    if target.exists() {
        return Err(format!(
            "'{store_relative_path}' already exists. Choose a different name — existing entries are never overwritten."
        ));
    }

    let mut command = Command::new("pass");
    command.arg("generate");
    if no_symbols {
        command.arg("-n");
    }
    command.arg(&store_relative_path);
    command.arg(length.to_string());

    let output = command
        .output()
        .map_err(|e| format!("Failed to run pass generate: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error generating password: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let password_raw = stdout
        .lines()
        .nth(1)
        .ok_or_else(|| "pass generate did not return a password".to_string())?
        .to_string();

    let plain_bytes = strip_ansi_escapes::strip(password_raw);
    let password = String::from_utf8_lossy(&plain_bytes).to_string();

    Ok(password)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let show = MenuItemBuilder::new("Show").id("show").build(app)?;
            let hide = MenuItemBuilder::new("Hide").id("hide").build(app)?;

            let menu = MenuBuilder::new(app).items(&[&quit, &show, &hide]).build()?;

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("missing default window icon")?;

            let _ = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                })
                .build(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize,
            load_password_store,
            copy_text_to_clipboard,
            copy_encrypted_password_to_clipboard,
            open_dialog,
            generate_new_pass
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_line_returns_password_only() {
        assert_eq!(first_line("hunter2\nusername: me\nurl: x\n"), "hunter2");
        assert_eq!(first_line("solo-password"), "solo-password");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn search_skips_hidden_and_non_gpg_files() {
        let dir = env::temp_dir().join(format!("pastool-gpg-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("keep.gpg"), b"x").unwrap();
        fs::write(dir.join("note.txt"), b"x").unwrap();
        fs::write(dir.join(".hidden.gpg"), b"x").unwrap();
        fs::write(dir.join("sub").join("nested.gpg"), b"x").unwrap();

        let found = search_for_gpg_files(&dir).unwrap();
        let mut rel: Vec<String> = found
            .iter()
            .map(|p| {
                Path::new(p)
                    .strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        rel.sort();

        assert_eq!(rel, vec!["keep.gpg".to_string(), "sub/nested.gpg".to_string()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn search_errors_on_unreadable_root() {
        let missing = env::temp_dir().join("pastool-missing-dir-xyz");
        let _ = fs::remove_dir_all(&missing);
        assert!(search_for_gpg_files(&missing).is_err());
    }

    #[test]
    fn newer_copy_supersedes_older_scheduled_clear() {
        // Re-copying the same secret within the 45s window must restart the
        // window: the older timer may not clear what the newer copy just wrote.
        let first = next_clipboard_clear_generation();
        assert!(is_current_clipboard_generation(first));

        let second = next_clipboard_clear_generation();
        assert!(second > first);
        assert!(!is_current_clipboard_generation(first));
        assert!(is_current_clipboard_generation(second));
    }

    #[test]
    fn generate_refuses_to_overwrite_existing_entry() {
        let home = env::temp_dir().join(format!("pastool-home-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(home.join(".password-store")).unwrap();
        fs::write(home.join(".password-store").join("existing.gpg"), b"x").unwrap();

        let previous_home = env::var_os("HOME");
        env::set_var("HOME", &home);

        let result = generate_new_pass("existing".to_string(), 25, false);

        match previous_home {
            Some(value) => env::set_var("HOME", value),
            None => env::remove_var("HOME"),
        }
        let _ = fs::remove_dir_all(&home);

        let err = result.expect_err("existing entry should not be overwritten");
        assert!(err.contains("already exists"));
    }
}
