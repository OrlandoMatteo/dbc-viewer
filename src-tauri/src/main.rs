// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use std::env;
// import the functions from can/engine.rs
mod can;
mod parser;

use crate::can::signals::get_card_from_signal;
use crate::can::signals::get_li_from_signal;
use crate::can::signals::get_signals;
use crate::can::signals::search_signal;
use crate::can::signals::search_signals;
use crate::can::signals::Signal;
use std::sync::Mutex;

use crate::can::messages::get_card_from_message;
use crate::can::messages::get_li_from_message;
use crate::can::messages::get_messages;
use crate::can::messages::search_message;
use crate::can::messages::search_messages_by_name;
use crate::can::messages::search_messages_by_signal;
use crate::can::messages::Message;

use crate::parser::parser::parse_dbc;

// Create a struct to hold the index and signals
struct AppState {
    signals: Mutex<Vec<Signal>>,
    messages: Mutex<Vec<Message>>,
}
impl AppState {
    fn new(signals: Vec<Signal>, messages: Vec<Message>) -> Self {
        Self {
            signals: Mutex::new(signals),
            messages: Mutex::new(messages),
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
    let signals_in_message: Vec<Message> = search_messages_by_signal(&messages, &query);
    // format the results to an html list
    let mut html = String::from("<ul class=\"list-group\">");
    for result in signal_result.iter() {
        html.push_str(&format!("{}", get_li_from_signal(result)));
    }
    for result in message_result.iter() {
        html.push_str(&format!("{}", get_li_from_message(result)));
    }
    for result in signals_in_message.iter() {
        html.push_str(&format!("{}", get_li_from_message(result)));
    }
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
fn upload_dbc(base64_data: String, app_state: tauri::State<AppState>) -> String {
    // Make the HTTP request in an asynchronous context
    let bytes = general_purpose::STANDARD.decode(base64_data).unwrap();
    let s = match String::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    let (messages, signals) = parse_dbc(&s);
    let mut state_mex = app_state.messages.lock().unwrap();
    let mut state_sig = app_state.signals.lock().unwrap();
    *state_mex = messages;
    *state_sig = signals;
    s
}

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {:?}", current_dir);
    // Create the index
    let json_file = "../sample/dbc.json";
    let json = std::fs::read_to_string(json_file).expect("Failed to read file");
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
            upload_dbc
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
