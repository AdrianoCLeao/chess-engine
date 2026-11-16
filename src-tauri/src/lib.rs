//! Chess Engine library: rules, search, Elo bots, analysis, coach mode, and Tauri API.
//!
//! Code documentation is in English. UI strings are Portuguese via [`i18n`], with
//! the locale system ready for additional languages.

pub mod api;
pub mod engine;
pub mod i18n;

use api::SharedState;
use std::sync::Mutex;

/// Start the Tauri desktop application.
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(api::AppState::default()) as SharedState)
        .invoke_handler(tauri::generate_handler![
            api::get_state,
            api::new_game,
            api::set_config,
            api::legal_targets,
            api::play_move,
            api::engine_move,
            api::analyze_current,
            api::list_elo_levels,
            api::elo_profile,
            api::methodology_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
