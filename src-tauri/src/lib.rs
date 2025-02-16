use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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
async fn copy_encrypted_password_to_clipboard(relative_path: String) -> Result<String, String> {
    // let mut command = Command::new("gpg");
    // command.arg("--decrypt");
    // command.arg(full_path);
    // command.arg("-o");
    // command.arg("-");

    println!("relative_path: {}", relative_path);

    let mut command = Command::new("pass");
    command.arg(relative_path);

    let output = command.output().unwrap();
    println!("output: {:?}", output);

    if output.status.success() {
        let output = String::from_utf8(output.stdout).unwrap();
        Ok(output)
    } else {
        Err(format!("Error decrypting password: {}", output.status))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_password_store,
            copy_encrypted_password_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
