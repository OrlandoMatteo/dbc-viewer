// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose, Engine as _};
use std::env;
use std::sync::Mutex;
// import the functions from can/engine.rs
mod can;
mod parser;

use crate::can::signals::*;

use crate::can::messages::get_card_from_message;
use crate::can::messages::get_li_from_message;
use crate::can::messages::search_message;
use crate::can::messages::search_messages_by_name;
use crate::can::messages::Message;

use crate::parser::parser::parse_dbc;

use serde_json::json;

// Create a struct to hold the index and signals
struct AppState {
    signals: Mutex<Vec<Signal>>,
    messages: Mutex<Vec<Message>>,
    filename: Mutex<String>,
}
impl AppState {
    fn new(signals: Vec<Signal>, messages: Vec<Message>) -> Self {
        Self {
            signals: Mutex::new(signals),
            messages: Mutex::new(messages),
            filename: Mutex::from(String::from("")),
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("pippo");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn search(query: &str, app_state: tauri::State<AppState>) -> String {
    println!("Searching for: {}", query);
    let signals = app_state.signals.lock().unwrap();
    let signal_result: Vec<Signal> = search_signals(&signals, &query);
    println!("FOUND: {:?}  signals", signal_result.len());
    let messages = app_state.messages.lock().unwrap();
    let message_result: Vec<Message> = search_messages_by_name(&messages, &query);
    println!("FOUND: {:?}  signals", signal_result.len());
    // format the results to an html list
    let mut html = String::from("<ul class=\"list-group\">");
    for result in signal_result.iter() {
        html.push_str(&format!("{}", get_li_from_signal(result)));
    }
    for result in message_result.iter() {
        html.push_str(&format!("{}", get_li_from_message(result)));
    }
    html.push_str("</ul>");
    html
}

#[tauri::command]
fn show_signal(query: &str, app_state: tauri::State<AppState>) -> String {
    let signals = app_state.signals.lock().unwrap();
    let result = search_signal(&signals, &query);
    println!("Signal: {:?}", result);
    match result {
        Some(signal) => format!("{}", get_card_from_signal(&signal)),
        None => "Signal not found".to_string(),
    }
}

#[tauri::command]
fn show_message(query: &str, app_state: tauri::State<AppState>) -> String {
    let messages = app_state.messages.lock().unwrap();
    let result = search_message(&messages, &query);
    println!("Messages: {:?}", result);
    match result {
        Some(message) => format!("{}", get_card_from_message(&message)),
        None => "Signal not found".to_string(),
    }
}
#[tauri::command]
fn upload_dbc(base64_data: String, filename: String, app_state: tauri::State<AppState>) -> String {
    // Make the HTTP request in an asynchronous context
    let bytes = general_purpose::STANDARD.decode(base64_data).unwrap();
    let response = match String::from_utf8(bytes) {
        Ok(v) => {
            let (messages, signals) = parse_dbc(&v);
            let mut state_mex = app_state.messages.lock().unwrap();
            let mut state_sig = app_state.signals.lock().unwrap();
            let mut state_filename = app_state.filename.lock().unwrap();
            *state_filename = filename;
            *state_mex = messages;
            *state_sig = signals;
            let response = json!({
            "code":200,
            "message":String::from(format!("Loaded file {}", state_filename))
            });
            response
        }
        Err(e) => {
            let response = json!({
            "code":400,
            "message":String::from(format!("Invalid UTF-8 sequence: {}", e))
            });
            response
        }
    };
    response.to_string()
}

#[tauri::command]
fn is_dbc_loaded(app_state: tauri::State<AppState>) -> String {
    if app_state.messages.lock().unwrap().len() == 0 {
        let response = json!({
        "code":404,
        "message":"No DBC loaded"
        });
        response.to_string()
    } else {
        let response = json!({
        "code":200,
        "message":String::from(format!("Loaded file {}",app_state.filename.lock().unwrap().clone()))
        });
        response.to_string()
    }
}

#[tauri::command]
fn get_all_signals(app_state: tauri::State<AppState>) -> String {
    let state_sig = app_state.signals.lock().unwrap();
    let mut html = String::from("<div class=\"accordion\" id=\"signalsAccordion\">");
    if state_sig.len() > 0 {
        for result in state_sig.iter() {
            html.push_str(&format!("{}", get_details_from_signal(result)));
        }
    }
    html.push_str("</div>");
    html
}
#[tauri::command]
fn get_all_messages(app_state: tauri::State<AppState>) -> String {
    let state_mex = app_state.messages.lock().unwrap();
    let mut html = String::from("<ul class=\"list-group\">");
    if state_mex.len() > 0 {
        for result in state_mex.iter() {
            html.push_str(&format!("{}", get_li_from_message(result)));
        }
    }
    html.push_str("</ul>");
    html
}

fn main() {
    // Create the index
    let signals = Vec::new();
    //let signals = get_signals(&json);
    let messages = Vec::new();
    //let messages = get_messages(&json);
    let app_state = AppState::new(signals, messages);
    println!("App state created");

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
