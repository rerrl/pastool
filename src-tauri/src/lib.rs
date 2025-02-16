use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
async fn load_password_store() -> Result<Vec<String>, String> {
    // Use the home directory to construct the path
    let home_dir =
        env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    let password_store = home_dir.join(".password-store");

    // Check if the path exists and is a directory
    if !password_store.exists() || !password_store.is_dir() {
        return Err(format!(
            "The password store directory '{}' does not exist or is not a directory",
            password_store.display()
        ));
    }

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app).unwrap();
            let hide = MenuItemBuilder::new("Hide").id("hide").build(app).unwrap();
            let show = MenuItemBuilder::new("Show").id("show").build(app).unwrap();
            // we could opt handle an error case better than calling unwrap
            let menu = MenuBuilder::new(app)
                .items(&[&quit, &hide, &show])
                .build()
                .unwrap();

            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "hide" => {
                        dbg!("menu item hide clicked");
                        let window = app.get_webview_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "show" => {
                        dbg!("menu item show clicked");
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                    }
                    _ => {}
                })
                // .on_tray_icon_event(|tray_icon, event| match event.click_type {
                //     ClickType::Left => {
                //         dbg!("system tray received a left click");
                //         let window = tray_icon.app_handle().get_webview_window("main").unwrap();
                //         let _ = window.show().unwrap();
                //         let logical_size = tauri::LogicalSize::<f64> {
                //             width: 300.00,
                //             height: 400.00,
                //         };
                //         let logical_s = tauri::Size::Logical(logical_size);
                //         let _ = window.set_size(logical_s);
                //         let logical_position = tauri::LogicalPosition::<f64> {
                //             x: event.x - logical_size.width,
                //             y: event.y - logical_size.height - 70.,
                //         };
                //         let logical_pos: tauri::Position =
                //             tauri::Position::Logical(logical_position);
                //         let _ = window.set_position(logical_pos);
                //         let _ = window.set_focus();
                //     }
                //     ClickType::Right => {
                //         dbg!("system tray received a right click");
                //         let window = tray_icon.app_handle().get_webview_window("main").unwrap();
                //         window.hide().unwrap();
                //     }
                //     ClickType::Double => {
                //         dbg!("system tray received a double click");
                //     }
                // })
                .build(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_password_store,
            copy_encrypted_password_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
