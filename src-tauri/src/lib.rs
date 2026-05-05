pub mod commands;
pub mod models;
pub mod services;
pub mod state;

#[cfg(test)]
mod tests;

pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_urls,
            commands::create_download_task,
            commands::get_settings,
            commands::save_settings,
            commands::list_history,
            commands::search_history,
            commands::remove_history_entry,
            commands::clear_history,
            commands::tail_logs,
            commands::get_queue_snapshot,
            commands::clear_queue,
            commands::pause_task,
            commands::retry_task,
            commands::get_tool_status,
            commands::get_update_status
        ])
        .setup(|app| {
            let _ = app.handle();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
