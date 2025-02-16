use std::fs;
use std::env;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PasswordStoreEntry {
    path: String,
    is_folder: bool,
    is_encrypted: bool,
}

#[tauri::command]
async fn load_password_store() -> Result<Vec<PasswordStoreEntry>, String> {
    // Use the home directory to construct the path
    let home_dir = env::home_dir().ok_or_else(|| "Failed to retrieve home directory".to_string())?;
    let password_store = home_dir.join(".password-store/");

    // log the path
    println!("password store path: {}", password_store.display());

    // Check if the path exists and is a directory
    if !password_store.exists() || !password_store.is_dir() {
        return Err(format!("The password store directory '{}' does not exist or is not a directory", password_store.display()));
    }

    let mut result = Vec::new();
    match fs::read_dir(&password_store) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();

                        // ignore hidden files and folders
                        if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
                            continue;
                        }

                        // Check if the entry is a directory
                        if path.is_dir() {
                            result.push(PasswordStoreEntry {
                                path: path.display().to_string(),
                                is_folder: true,
                                is_encrypted: false,
                            });
                        } else {
                            // Check if the entry is a file
                            if path.is_file() {
                                // Check if the file is encrypted (ends with .gpg)
                                if let Some(ext) = path.extension() {
                                    if ext == "gpg" {
                                        result.push(PasswordStoreEntry {
                                            path: path.display().to_string(),
                                            is_folder: false,
                                            is_encrypted: true,
                                        });
                                    }
                                }
                            } // else we just ignore it
                        }
                    },
                    Err(e) => return Err(format!("Error reading directory entry: {}", e)),
                }
            }
        },
        Err(e) => return Err(format!("Error reading directory: {}", e)),
    }
    Ok(result)
    
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_password_store])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
