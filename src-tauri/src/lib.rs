use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

/// How long a copied secret stays on the clipboard before we try to clear it.
const CLIPBOARD_CLEAR_SECONDS: u64 = 45;

#[derive(serde::Serialize, serde::Deserialize)]
struct SystemStatus {
    has_gpg: bool,
    has_pass: bool,
    has_pass_store: bool,
    home_dir: String,
    /// Detected package manager for install instructions: "apt", "dnf",
    /// "pacman", or "unknown".
    pkg_manager: String,
}

/// A secret (private) GPG key the user can encrypt the password store with.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct GpgKey {
    /// Long key id (colons format field 5 of the `sec` record).
    id: String,
    /// Human readable "Name <email>" user id.
    uid: String,
}

/// Map the contents of `/etc/os-release` to the package manager we should
/// suggest. Pure so it can be unit tested without touching the real system.
fn package_manager_from_os_release(content: &str) -> &'static str {
    let mut hay = String::new();
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            hay.push_str(&value.trim_matches('"').to_lowercase());
            hay.push(' ');
        } else if let Some(value) = line.strip_prefix("ID_LIKE=") {
            hay.push_str(&value.trim_matches('"').to_lowercase());
            hay.push(' ');
        }
    }

    for token in hay.split_whitespace() {
        match token {
            "debian" | "ubuntu" | "linuxmint" | "pop" | "kali" | "raspbian" | "zorin"
            | "elementary" | "deepin" => return "apt",
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" | "ol" => return "dnf",
            "arch" | "manjaro" | "endeavouros" | "cachyos" | "garuda" | "artix" => {
                return "pacman"
            }
            _ => {}
        }
    }

    "unknown"
}

/// Detect the package manager, first from `/etc/os-release`, then by probing
/// for the manager binaries themselves.
fn detect_package_manager() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let detected = package_manager_from_os_release(&content);
        if detected != "unknown" {
            return detected.to_string();
        }
    }

    for (binary, name) in [("apt-get", "apt"), ("dnf", "dnf"), ("pacman", "pacman")] {
        if Command::new(binary).arg("--version").output().is_ok() {
            return name.to_string();
        }
    }

    "unknown".to_string()
}

/// Parse `gpg --list-secret-keys --with-colons` output into our keys. Pure so
/// it can be tested without a keyring.
fn parse_secret_keys(colons: &str) -> Vec<GpgKey> {
    let mut keys: Vec<GpgKey> = Vec::new();

    for line in colons.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        match fields.first().copied() {
            Some("sec") => {
                // Field 5 (index 4) is the long key id.
                let id = fields.get(4).copied().unwrap_or("").to_string();
                if !id.is_empty() {
                    keys.push(GpgKey {
                        id,
                        uid: String::new(),
                    });
                }
            }
            Some("uid") => {
                // Field 10 (index 9) is the user id string. Attach it to the
                // most recent secret key.
                if let Some(key) = keys.last_mut() {
                    if key.uid.is_empty() {
                        key.uid = fields.get(9).copied().unwrap_or("").to_string();
                    }
                }
            }
            _ => {}
        }
    }

    keys.retain(|key| !key.id.is_empty());
    keys
}

/// Build the `gpg --batch --gen-key` parameter file. ed25519 signing key with
/// an cv25519 encryption subkey, no expiry, and `%no-protection` so the user is
/// never asked for a passphrase (and we never pass one on the command line).
fn gen_key_params(name: &str, email: &str) -> String {
    format!(
        "Key-Type: eddsa\n\
         Key-Curve: ed25519\n\
         Key-Usage: sign cert\n\
         Subkey-Type: ecdh\n\
         Subkey-Curve: cv25519\n\
         Subkey-Usage: encrypt\n\
         Name-Real: {name}\n\
         Name-Email: {email}\n\
         Expire-Date: 0\n\
         %no-protection\n\
         %commit\n"
    )
}

/// Strip characters that could break out of a single parameter-file line.
fn sanitize_param_value(value: &str) -> String {
    value.replace(['\n', '\r'], " ").trim().to_string()
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
fn schedule_clipboard_clear(app: &tauri::AppHandle, expected: String) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(CLIPBOARD_CLEAR_SECONDS));

        if let Ok(current) = app.clipboard().read_text() {
            if current == expected {
                let _ = app.clipboard().write_text(String::new());
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
        pkg_manager: detect_package_manager(),
    })
}

/// List the secret keys in the user's keyring. An empty list means the user has
/// no key yet. Missing gpg is an error the wizard shows inline.
#[tauri::command]
fn list_secret_keys() -> Result<Vec<GpgKey>, String> {
    let output = Command::new("gpg")
        .args(["--list-secret-keys", "--with-colons"])
        .output()
        .map_err(|e| format!("gpg is not installed or could not be run: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let keys = parse_secret_keys(&stdout);

    // gpg exits non-zero in some setups even when keys are listed, so only
    // treat it as an error when we got nothing useful back.
    if keys.is_empty() && !output.status.success() && stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Could not read your GPG keys: {}", stderr.trim()));
    }

    Ok(keys)
}

/// Create a new GPG key for the user with `gpg --batch --gen-key`, passing the
/// parameter file on stdin (never a passphrase on the command line). Returns
/// the key that was created.
#[tauri::command]
fn generate_gpg_key(name: String, email: String) -> Result<GpgKey, String> {
    let name = sanitize_param_value(&name);
    let email = sanitize_param_value(&email);

    if name.is_empty() {
        return Err("Please enter your name.".to_string());
    }
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err("Please enter a valid email address.".to_string());
    }

    let params = gen_key_params(&name, &email);

    let mut child = Command::new("gpg")
        .args(["--batch", "--gen-key"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gpg is not installed or could not be run: {e}"))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "Could not talk to gpg.".to_string())?;
        stdin
            .write_all(params.as_bytes())
            .map_err(|e| format!("Could not talk to gpg: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("gpg did not finish: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        return Err(if detail.is_empty() {
            "gpg could not create a key. Please try again.".to_string()
        } else {
            format!("gpg could not create a key: {detail}")
        });
    }

    // Find the key we just made by matching the email we were given.
    let keys = list_secret_keys()?;
    let email_lower = email.to_lowercase();
    let created = keys
        .iter()
        .find(|key| key.uid.to_lowercase().contains(&email_lower))
        .or_else(|| keys.last())
        .cloned()
        .ok_or_else(|| {
            "gpg finished but no key was found afterwards. Please try again.".to_string()
        })?;

    Ok(created)
}

/// Initialize the password store with `pass init <key id>`. This creates
/// `~/.password-store` and the `.gpg-id` file that tells pass which key to use.
#[tauri::command]
fn init_pass_store(key_id: String) -> Result<(), String> {
    let key_id = key_id.trim().to_string();
    if key_id.is_empty() {
        return Err("Please choose a GPG key first.".to_string());
    }

    let output = Command::new("pass")
        .arg("init")
        .arg(&key_id)
        .output()
        .map_err(|e| format!("pass is not installed or could not be run: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        return Err(if detail.is_empty() {
            "pass could not set up your password store. Please try again.".to_string()
        } else {
            format!("pass could not set up your password store: {detail}")
        });
    }

    // Make sure the store is where the rest of the app expects it.
    if !password_store_dir()?.is_dir() {
        return Err("pass finished but the password store folder is missing.".to_string());
    }

    Ok(())
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
            list_secret_keys,
            generate_gpg_key,
            init_pass_store,
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

    #[test]
    fn os_release_maps_to_package_manager() {
        assert_eq!(
            package_manager_from_os_release(
                "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"24.04\"\n"
            ),
            "apt"
        );
        assert_eq!(
            package_manager_from_os_release("NAME=\"Debian\"\nID=debian\n"),
            "apt"
        );
        assert_eq!(
            package_manager_from_os_release("ID=fedora\nID_LIKE=\"rhel\"\n"),
            "dnf"
        );
        assert_eq!(
            package_manager_from_os_release("ID=manjaro\nID_LIKE=arch\n"),
            "pacman"
        );
        assert_eq!(package_manager_from_os_release("ID=void\n"), "unknown");
        assert_eq!(package_manager_from_os_release(""), "unknown");
    }

    #[test]
    fn parses_secret_keys_from_colons_output() {
        let colons = "sec:u:255:22:C4FCE780123E8567:1794676087:::u:::scESC:::+::ed25519:::0:\n\
                      fpr:::::::::3DB429A639E008446D1557DFC4FCE780123E8567:\n\
                      grp:::::::::776D306EC79991284F726282FD4A72F4CDABF96F:\n\
                      uid:u::::1794676087::F2E723F89EE13F7007A48DDACDB395768100ABC3::Test User <test@example.com>::::::::::0:\n\
                      ssb:u:255:18:996E64BFF98F02EB:1794676087::::::e:::+::cv25519::\n\
                      sec:u:255:22:AAAA1111BBBB2222:1794676088:::u:::scESC:::+::ed25519:::0:\n\
                      uid:u::::1794676088::DEADBEEF::Second Person <two@example.com>::::::::::0:\n";

        let keys = parse_secret_keys(colons);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].id, "C4FCE780123E8567");
        assert_eq!(keys[0].uid, "Test User <test@example.com>");
        assert_eq!(keys[1].id, "AAAA1111BBBB2222");
        assert_eq!(keys[1].uid, "Second Person <two@example.com>");
    }

    #[test]
    fn no_secret_keys_is_empty_not_error() {
        assert!(parse_secret_keys("").is_empty());
        assert!(parse_secret_keys("\n\n").is_empty());
    }

    #[test]
    fn gen_key_params_are_batch_safe() {
        let params = gen_key_params("Ada Lovelace", "ada@example.com");
        assert!(params.contains("Name-Real: Ada Lovelace\n"));
        assert!(params.contains("Name-Email: ada@example.com\n"));
        assert!(params.contains("Key-Curve: ed25519\n"));
        assert!(params.contains("%no-protection\n"));
        assert!(params.ends_with("%commit\n"));
        // No passphrase is ever supplied.
        assert!(!params.to_lowercase().contains("passphrase"));
        assert!(!params.contains("%ask-passphrase"));
    }

    #[test]
    fn sanitize_strips_newlines() {
        assert_eq!(sanitize_param_value("Ada\nLovelace"), "Ada Lovelace");
        assert_eq!(sanitize_param_value("  spaced  "), "spaced");
        assert_eq!(sanitize_param_value("a\r\nb"), "a  b");
    }
}
