mod near_get_greeting;
mod near_credentials;
mod near_update_greeting;
use near_get_greeting::get_near_greeting;
use near_credentials::load_near_credentials;
use near_update_greeting::update_near_greeting;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_near_greeting, load_near_credentials, update_near_greeting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
