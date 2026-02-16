mod cli;
mod git;
mod merge;
mod safety;
mod state;

use std::fs;
use std::path::Path;
use tauri::AppHandle;

use merge::hunk::{HunkStatus, MergeSession};
use merge::three_way::three_way_merge;
use merge::resolver;
use state::AppState;

/// Detect language from file extension.
fn detect_language(path: &str) -> String {
    let ext = Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        "html" | "htm" => "html",
        "css" => "css",
        "sh" | "bash" | "zsh" => "shell",
        "sql" => "sql",
        "xml" => "xml",
        _ => "plaintext",
    }
    .to_string()
}

/// Parse conflict markers from an already-merged file (fallback mode).
fn parse_conflict_markers(content: &str) -> Option<Vec<(String, String)>> {
    if !content.contains("<<<<<<<") {
        return None;
    }

    let mut conflicts = Vec::new();
    let mut local_lines = Vec::new();
    let mut remote_lines = Vec::new();
    let mut in_local = false;
    let mut in_remote = false;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_local = true;
            local_lines.clear();
            remote_lines.clear();
        } else if line.starts_with("=======") {
            in_local = false;
            in_remote = true;
        } else if line.starts_with(">>>>>>>") {
            in_remote = false;
            conflicts.push((local_lines.join("\n"), remote_lines.join("\n")));
        } else if in_local {
            local_lines.push(line.to_string());
        } else if in_remote {
            remote_lines.push(line.to_string());
        }
    }

    if conflicts.is_empty() {
        None
    } else {
        Some(conflicts)
    }
}

#[tauri::command]
fn get_merge_session(state: tauri::State<'_, AppState>) -> Result<MergeSession, String> {
    let guard = state.session.lock();
    guard.clone().ok_or_else(|| "No merge session initialized".to_string())
}

#[tauri::command]
fn resolve_hunk(
    hunk_id: usize,
    status: HunkStatus,
    state: tauri::State<'_, AppState>,
) -> Result<MergeSession, String> {
    let mut guard = state.session.lock();
    let session = guard.as_mut().ok_or("No merge session")?;
    resolver::resolve_hunk(session, hunk_id, status);
    Ok(session.clone())
}

#[tauri::command]
fn save_result(content: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let guard = state.session.lock();
    let session = guard.as_ref().ok_or("No merge session")?;
    let path = Path::new(&session.merged_path);
    safety::safe_write(path, &content)
}

#[tauri::command]
fn update_result_content(content: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.session.lock();
    let session = guard.as_mut().ok_or("No merge session")?;
    session.result_content = content;
    Ok(())
}

#[tauri::command]
fn abort_merge(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.aborted.lock() = true;
    app.exit(1);
    Ok(())
}

#[tauri::command]
fn auto_resolve(state: tauri::State<'_, AppState>) -> Result<MergeSession, String> {
    let mut guard = state.session.lock();
    let session = guard.as_mut().ok_or("No merge session")?;
    resolver::auto_resolve_non_conflicts(session);
    merge::structural::structural_auto_resolve(session);
    Ok(session.clone())
}

#[tauri::command]
fn register_git_mergetool() -> Result<String, String> {
    git::repo::register_mergetool()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    // Parse CLI args and initialize session
    if let Some(args) = cli::CliArgs::parse() {
        let base_content = fs::read_to_string(&args.base)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Could not read base file: {}", e);
                String::new()
            });
        let local_content = fs::read_to_string(&args.local)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Could not read local file: {}", e);
                String::new()
            });
        let remote_content = fs::read_to_string(&args.remote)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Could not read remote file: {}", e);
                String::new()
            });
        let merged_content = fs::read_to_string(&args.merged).unwrap_or_default();

        let language = detect_language(&args.merged.to_string_lossy());

        // Check if the merged file already has conflict markers (fallback mode)
        let (hunks, result_content) = if !base_content.is_empty() {
            three_way_merge(&base_content, &local_content, &remote_content)
        } else if parse_conflict_markers(&merged_content).is_some() {
            // Fallback: parse conflict markers from the merged file
            // Use merged as base and reconstruct
            three_way_merge(&merged_content, &local_content, &remote_content)
        } else {
            (Vec::new(), local_content.clone())
        };

        let session = MergeSession {
            base_path: args.base.to_string_lossy().to_string(),
            local_path: args.local.to_string_lossy().to_string(),
            remote_path: args.remote.to_string_lossy().to_string(),
            merged_path: args.merged.to_string_lossy().to_string(),
            base_content,
            local_content,
            remote_content,
            result_content,
            hunks,
            language,
        };

        *app_state.session.lock() = Some(session);
    }

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_merge_session,
            resolve_hunk,
            save_result,
            update_result_content,
            abort_merge,
            auto_resolve,
            register_git_mergetool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
