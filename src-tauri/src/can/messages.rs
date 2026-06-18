use std::num::ParseIntError;

use crate::can::signals::search_signal;
use crate::can::signals::Signal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    #[serde(default = "default_u64")]
    pub can_id: u64,
    pub pgn: u64,
    pub source: u16,
    pub name: String,
    pub priority: u16,
    pub label: String,
    #[serde(rename = "isExtendedFrame")]
    pub is_extended_frame: bool,
    pub dlc: u16,
    pub comment: Option<String>,
    #[serde(default = "default_i64")]
    pub line_in_dbc: i64,
    pub problems: Vec<Problem>,
    pub signals: Vec<String>,
}
fn default_u64() -> u64 {
    0
}
fn default_i64() -> i64 {
    0
}
impl Message {
    pub fn new() -> Message {
        Message {
            can_id: 0,
            pgn: 0,
            source: 0,
            name: String::from(""),
            priority: 0,
            label: String::from(""),
            is_extended_frame: false,
            dlc: 0,
            comment: Some(String::from("")),
            line_in_dbc: 0,
            problems: Vec::new(),
            signals: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Problem {
    severity: String,
    line: usize,
    description: String,
}

pub fn search_messages_by_name(messages: &[Message], query: &str) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    for i in messages.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase().contains(&query.to_lowercase()) {
            result.push(i.clone());
        }
    }
    result
}
pub fn search_messages_by_id(messages: &[Message], query: &str) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    match is_valid_hexadecimal(query) {
        Ok(hex_num) => {
            for i in messages.iter() {
                if i.can_id == hex_num {
                    result.push(i.clone());
                }
            }
        }
        Err(_) => {}
    }
    result
}
pub fn search_message(messages: &[Message], query: &str) -> Option<Message> {
    for i in messages.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase() == query.to_lowercase() {
            return Some(i.clone());
        }
    }
    None
}

pub fn resolve_message_signals(message: &Message, signals: &[Signal]) -> Vec<Signal> {
    message
        .signals
        .iter()
        .filter_map(|signal_name| search_signal(signals, signal_name))
        .collect()
}

fn is_valid_hexadecimal(s: &str) -> Result<u64, ParseIntError> {
    if s.len() > 2 && s.starts_with("0x") {
        // If the string starts with "0x", skip those characters
        u64::from_str_radix(&s[2..], 16)
    } else {
        // Otherwise, parse the string directly
        u64::from_str_radix(s, 16)
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_message_signals, Message};
    use crate::can::signals::Signal;

    #[test]
    fn missing_signal_references_resolve_to_empty_results() {
        let mut message = Message::new();
        message.signals.push("MissingSignal".to_string());
        let signals: Vec<Signal> = Vec::new();

        let resolved = resolve_message_signals(&message, &signals);

        assert!(resolved.is_empty());
    }
}
