use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::ipc::RuntimeCapability;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
// use tauri::tray::MouseButton;
// use tauri::tray::MouseButtonState;
// use tauri::tray::TrayIconEvent;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FileDialogBuilder;

#[derive(serde::Serialize, serde::Deserialize)]
struct SystemStatus {
    has_gpg: bool,
    has_pass: bool,
    has_pass_store: bool,
    home_dir: String,
}

#[tauri::command]
async fn initialize() -> Result<SystemStatus, String> {
    let home_dir =
        env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    let password_store = home_dir.join(".password-store");

    Ok(SystemStatus {
        has_gpg: Command::new("gpg").arg("--version").output().is_ok(),
        has_pass: Command::new("pass").arg("--version").output().is_ok(),
        has_pass_store: password_store.exists() && password_store.is_dir(),
        home_dir: home_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn load_password_store() -> Result<Vec<String>, String> {
    // Use the home directory to construct the path
    let home_dir =
        env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    let password_store = home_dir.join(".password-store");

    Ok(search_for_gpg_files(&password_store))
}

// functoin to recusievely search in directories for .gpg files
fn search_for_gpg_files(path: &Path) -> Vec<String> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // ignore hidden files and folders
        if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
            continue;
        }

        if path.is_dir() {
            result.extend(search_for_gpg_files(&path));
        } else {
            if path.extension().unwrap() == "gpg" {
                result.push(path.display().to_string());
            }
        }
    }

    result
}

#[tauri::command]
async fn copy_encrypted_password_to_clipboard(
    relative_path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let mut command = Command::new("pass");
    command.arg(relative_path);

    let output = command.output().unwrap();

    if output.status.success() {
        let output = String::from_utf8(output.stdout).unwrap();

        // use clipboard manager to copy to clipboard
        let _ = app.clipboard().write_text(output.clone());

        Ok("Copied to clipboard!".to_string())
    } else {
        Err(format!("Error decrypting password: {}", output.status))
    }
}

#[tauri::command]
async fn open_dialog(app: tauri::AppHandle) -> Result<String, String> {
    let home_dir =
        env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    let password_store = home_dir.join(".password-store");

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app).unwrap();
            let show = MenuItemBuilder::new("Show").id("show").build(app).unwrap();
            let hide = MenuItemBuilder::new("Hide").id("hide").build(app).unwrap();

            // we could opt handle an error case better than calling unwrap
            let menu = MenuBuilder::new(app)
                .items(&[&quit, &show, &hide])
                .build()
                .unwrap();

            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        dbg!("menu item show clicked");
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                    }
                    "hide" => {
                        dbg!("menu item hide clicked");
                        let window = app.get_webview_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    _ => {
                        dbg!("menu item unknown clicked");
                    }
                })
                // .on_tray_icon_event(|tray, event: TrayIconEvent| match event {
                //     TrayIconEvent::Click {
                //         button: MouseButton::Left,
                //         button_state: MouseButtonState::Up,
                //         ..
                //     } => {
                //         println!("left click pressed and released");
                //         // in this example, let's show and focus the main window when the tray is clicked
                //         let app = tray.app_handle();
                //         if let Some(window) = app.get_webview_window("main") {
                //             let _ = window.show();
                //             let _ = window.set_focus();
                //         }
                //     }
                //     _ => {
                //         println!("unhandled event {event:?}");
                //     }
                // })
                .build(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize,
            load_password_store,
            copy_encrypted_password_to_clipboard,
            open_dialog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
