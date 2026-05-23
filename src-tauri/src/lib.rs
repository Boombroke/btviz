mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::LiveSession::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_xml,
            commands::connect_server,
            commands::disconnect_server,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
