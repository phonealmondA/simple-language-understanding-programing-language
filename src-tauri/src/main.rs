// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use quantum_slut_transpiler::tauri_commands;

fn main() {
    tauri::Builder::default()
        .manage(tauri_commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::run_file,
            tauri_commands::get_cache_stats,
            tauri_commands::run_until_solved,
            tauri_commands::stop_execution,
            tauri_commands::reset_transpiler,
            tauri_commands::get_working_directory,
            tauri_commands::get_cache_history,
            tauri_commands::clear_memory_state,
            // Pattern Learning Commands
            tauri_commands::get_pattern_learning_data,
            tauri_commands::get_pattern_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
