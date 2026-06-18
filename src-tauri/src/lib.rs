//! Tauri command layer. All validation logic lives in `dms-core`; this is a
//! thin adapter plus unrestricted local file read/write for the open/save UI.

use dms_core::{RuleDoc, ValidationReport};

/// Validate a table-mapping document. `target_dms_version` gates version
/// warnings (SPEC §7.4).
#[tauri::command]
fn validate(content: String, target_dms_version: Option<String>) -> ValidationReport {
    dms_core::validate(&content, target_dms_version.as_deref())
}

/// The full rule catalog (SPEC §6) for the in-app reference table.
#[tauri::command]
fn list_rules() -> Vec<RuleDoc> {
    dms_core::list_rules()
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validate,
            list_rules,
            read_text_file,
            write_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
