use base64::{engine::general_purpose, Engine as _};
use encoding_rs;
use serde::Serialize;
use std::collections::HashSet;
use std::env;
use std::sync::Mutex;
use tauri::Emitter;

mod can;
mod parser;

use crate::can::messages::{
    resolve_message_signals, search_message, search_messages_by_id, search_messages_by_name,
    Message,
};
use crate::can::signals::{search_signal, search_signals, search_signals_by_id, Signal};
use crate::parser::parser::parse_dbc;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadResponse {
    loaded: bool,
    filename: Option<String>,
    message: String,
    warnings: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    kind: String,
    name: String,
    id: String,
    label: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MessageDetail {
    message: Message,
    signals: Vec<Signal>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", content = "item", rename_all = "camelCase")]
enum ViewItem {
    Signal(Signal),
    Message(MessageDetail),
}

struct AppState {
    signals: Mutex<Vec<Signal>>,
    messages: Mutex<Vec<Message>>,
    filename: Mutex<Option<String>>,
    parse_warnings: Mutex<Vec<String>>,
    view_history: Mutex<Vec<ViewItem>>,
    history_index: Mutex<Option<usize>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            signals: Mutex::new(Vec::new()),
            messages: Mutex::new(Vec::new()),
            filename: Mutex::new(None),
            parse_warnings: Mutex::new(Vec::new()),
            view_history: Mutex::new(Vec::new()),
            history_index: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn search(query: &str, app_state: tauri::State<AppState>) -> Vec<SearchResult> {
    let query = query.trim();
    if query.len() < 3 {
        return Vec::new();
    }

    let signals = app_state.signals.lock().unwrap();
    let messages = app_state.messages.lock().unwrap();
    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for signal in search_signals(&signals, query)
        .into_iter()
        .chain(search_signals_by_id(&signals, query))
    {
        if seen.insert(format!("signal:{}", signal.name.to_lowercase())) {
            results.push(SearchResult {
                kind: "signal".to_string(),
                name: signal.name,
                id: signal.sig_id.to_string(),
                label: signal.label,
            });
        }
    }

    for message in search_messages_by_name(&messages, query)
        .into_iter()
        .chain(search_messages_by_id(&messages, query))
    {
        if seen.insert(format!("message:{}", message.name.to_lowercase())) {
            results.push(SearchResult {
                kind: "message".to_string(),
                name: message.name,
                id: format!("{:#X}", message.can_id),
                label: message.label,
            });
        }
    }

    results
}

#[tauri::command]
fn show_signal(query: &str, app_state: tauri::State<AppState>) -> Result<ViewItem, String> {
    let signals = app_state.signals.lock().unwrap();
    let signal = search_signal(&signals, query).ok_or_else(|| "Signal not found".to_string())?;
    let view = ViewItem::Signal(signal);
    push_history(&app_state, view.clone());
    Ok(view)
}

#[tauri::command]
fn show_message(query: &str, app_state: tauri::State<AppState>) -> Result<ViewItem, String> {
    let messages = app_state.messages.lock().unwrap();
    let signals = app_state.signals.lock().unwrap();
    let message =
        search_message(&messages, query).ok_or_else(|| "Message not found".to_string())?;
    let view = ViewItem::Message(message_detail(&message, &signals));
    push_history(&app_state, view.clone());
    Ok(view)
}

#[tauri::command]
fn upload_dbc(
    base64_data: String,
    filename: String,
    app_state: tauri::State<AppState>,
) -> LoadResponse {
    match general_purpose::STANDARD.decode(base64_data) {
        Ok(contents) => load_dbc_bytes(&contents, filename, &app_state),
        Err(error) => LoadResponse {
            loaded: false,
            filename: None,
            message: format!("Invalid DBC upload data: {}", error),
            warnings: Vec::new(),
        },
    }
}

#[tauri::command]
fn load_file_from_path(path: String, app_state: tauri::State<AppState>) -> LoadResponse {
    match std::fs::read(&path) {
        Ok(contents) => {
            let filename = std::path::Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            load_dbc_bytes(&contents, filename, &app_state)
        }
        Err(error) => LoadResponse {
            loaded: false,
            filename: None,
            message: format!("Failed to read file: {}", error),
            warnings: Vec::new(),
        },
    }
}

#[tauri::command]
fn is_dbc_loaded(app_state: tauri::State<AppState>) -> LoadResponse {
    let filename = app_state.filename.lock().unwrap().clone();
    let warnings = app_state.parse_warnings.lock().unwrap().clone();

    match filename {
        Some(filename) => LoadResponse {
            loaded: true,
            message: format!("Loaded file {}", filename),
            filename: Some(filename),
            warnings,
        },
        None => LoadResponse {
            loaded: false,
            filename: None,
            message: "No DBC loaded".to_string(),
            warnings,
        },
    }
}

#[tauri::command]
fn get_all_signals(app_state: tauri::State<AppState>) -> Vec<Signal> {
    app_state.signals.lock().unwrap().clone()
}

#[tauri::command]
fn get_all_messages(app_state: tauri::State<AppState>) -> Vec<MessageDetail> {
    let messages = app_state.messages.lock().unwrap();
    let signals = app_state.signals.lock().unwrap();
    messages
        .iter()
        .map(|message| message_detail(message, &signals))
        .collect()
}

#[tauri::command]
fn handle_history(query: &str, app_state: tauri::State<AppState>) -> Result<ViewItem, String> {
    let history = app_state.view_history.lock().unwrap();
    if history.is_empty() {
        return Err("No view history yet".to_string());
    }

    let mut index = app_state.history_index.lock().unwrap();
    let current = index.unwrap_or(0);
    let next = match query {
        "Prev" if current > 0 => current - 1,
        "Next" if current + 1 < history.len() => current + 1,
        _ => current,
    };
    *index = Some(next);

    Ok(history[next].clone())
}

fn load_dbc_bytes(
    contents: &[u8],
    filename: String,
    app_state: &tauri::State<AppState>,
) -> LoadResponse {
    let (decoded_string, _, _) = encoding_rs::WINDOWS_1252.decode(contents);
    let parsed = parse_dbc(decoded_string.as_ref());

    let message_count = parsed.messages.len();
    let signal_count = parsed.signals.len();
    let warnings = parsed.warnings;

    *app_state.messages.lock().unwrap() = parsed.messages;
    *app_state.signals.lock().unwrap() = parsed.signals;
    *app_state.filename.lock().unwrap() = Some(filename.clone());
    *app_state.parse_warnings.lock().unwrap() = warnings.clone();
    app_state.view_history.lock().unwrap().clear();
    *app_state.history_index.lock().unwrap() = None;

    LoadResponse {
        loaded: true,
        filename: Some(filename.clone()),
        message: format!(
            "Loaded file {} ({} messages, {} signals)",
            filename, message_count, signal_count
        ),
        warnings,
    }
}

fn message_detail(message: &Message, signals: &[Signal]) -> MessageDetail {
    MessageDetail {
        message: message.clone(),
        signals: resolve_message_signals(message, signals),
    }
}

fn push_history(app_state: &tauri::State<AppState>, view: ViewItem) {
    let mut history = app_state.view_history.lock().unwrap();
    history.push(view);
    *app_state.history_index.lock().unwrap() = Some(history.len() - 1);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            search,
            show_signal,
            show_message,
            upload_dbc,
            is_dbc_loaded,
            get_all_signals,
            get_all_messages,
            handle_history,
            load_file_from_path,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let args: Vec<String> = env::args().collect();

            if let Some(file_path) = args.get(1).cloned() {
                if file_path.to_lowercase().ends_with(".dbc") {
                    let filename = std::path::Path::new(&file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let _ = handle.emit(
                            "file-open",
                            serde_json::json!({
                                "path": file_path,
                                "filename": filename
                            }),
                        );
                    });
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
