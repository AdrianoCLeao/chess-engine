#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn greet(name: String, square: Option<String>) -> String {
    let square_info = square.unwrap_or_else(|| "nenhuma casa selecionada".to_string());
    format!("Olá, {}! Casa atual: {}", name, square_info)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
