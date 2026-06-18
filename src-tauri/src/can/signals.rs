use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Problem {
    severity: String,
    line: u32,
    description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct State {
    pub value: i32,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Signal {
    pub name: String,
    pub label: String,
    #[serde(rename = "startBit")]
    pub start_bit: u32,
    #[serde(rename = "bitLength")]
    pub bit_length: u32,
    #[serde(rename = "isLittleEndian")]
    pub is_little_endian: bool,
    #[serde(rename = "isSigned")]
    pub is_signed: bool,
    pub factor: f64,
    pub offset: f32,
    #[serde(default = "default_float")]
    pub min: f32,
    #[serde(default = "default_float")]
    pub max: f32,
    #[serde(rename = "sourceUnit")]
    #[serde(default = "String::new")]
    pub source_unit: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    pub choking: bool,
    pub visibility: bool,
    pub interval: u32,
    pub category: String,
    #[serde(rename = "lineInDbc")]
    pub line_in_dbc: u32,
    pub problems: Vec<Problem>,
    #[serde(rename = "postfixMetric")]
    #[serde(default = "String::new")]
    pub postfix_metric: String,
    #[serde(default = "Vec::new")]
    pub states: Vec<State>,
    #[serde(rename = "msgId")]
    pub msg_id: u64,
    #[serde(rename = "msgName")]
    pub msg_name: String,
    pub sig_id: i32,
}

fn default_float() -> f32 {
    0.0
}

// function to search the a vector of SignalItem according to the index of the SignalsIndexItem
// the input is a vector of int with the index of the SignalsIndexItem that match the search
// the function returns a vector of SignalItem that match the search

pub fn search_signals(signals: &[Signal], query: &str) -> Vec<Signal> {
    let mut result: Vec<Signal> = Vec::new();
    for i in signals.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase().contains(&query.to_lowercase()) {
            result.push(i.clone());
        }
    }
    result
}

pub fn search_signals_by_id(signals: &[Signal], query: &str) -> Vec<Signal> {
    let mut result: Vec<Signal> = Vec::new();
    match i32::from_str_radix(query, 10) {
        Ok(num) => {
            for i in signals.iter() {
                // if the name of the signal contains the query, ignore case
                if i.sig_id == num {
                    result.push(i.clone());
                }
            }
        }
        Err(_) => {}
    }

    result
}

// search a signal by its name
pub fn search_signal(signals: &[Signal], query: &str) -> Option<Signal> {
    for i in signals.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase() == query.to_lowercase() {
            return Some(i.clone());
        }
    }
    None
}
