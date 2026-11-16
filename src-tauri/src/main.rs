//! Chess Engine desktop application entry point (Tauri 2).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chess_engine::run();
}
