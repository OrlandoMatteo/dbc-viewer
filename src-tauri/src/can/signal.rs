pub struct Signal {
    pub name: String,
    pub label: String,
    pub start_bit: i64,
    pub bit_length: i64,
    pub factor: f64,
    pub offset: i64,
    pub min: i64,
    pub max: f64,
    pub source_unit: String,
    pub data_type: String,
    pub interval: i64,
    pub category: String,
    pub line_in_dbc: i64,
    pub states: String,
    pub msg_id: u64,
    pub msg_name: String,
}

impl Signal {
    pub fn new(
        name: String,
        label: String,
        start_bit: i64,
        bit_length: i64,
        factor: f64,
        offset: i64,
        min: i64,
        max: f64,
        source_unit: String,
        data_type: String,
        interval: i64,
        category: String,
        line_in_dbc: i64,
        states: String,
        msg_id: u64,
        msg_name: String,
    ) -> Signal {
        Signal {
            name,
            label,
            start_bit,
            bit_length,
            factor,
            offset,
            min,
            max,
            source_unit,
            data_type,
            interval,
            category,
            line_in_dbc,
            states,
            msg_id,
            msg_name,
        }
    }
}

pub struct SignalsItem {
    pub Signals: Vec<Signal>,
    pub id: u64,
}

pub struct SignalsIndexItem {
    pub name: String,
    pub id: u64,
}
// SignalsIndexItem impl
impl SignalsIndexItem {
    pub fn new(name: String, id: u64) -> SignalsIndexItem {
        SignalsIndexItem { name, id }
    }
}

pub fn get_signals(json: &str) -> Vec<SignalItem> {
    let mut signals: Vec<SignalItem> = Vec::new();
    let dbc_json: serde_json::Value = serde_json::from_str(json).expect("Failed to parse JSON");
    let signals_json = dbc_json.as_object().get("signals");
    for (id, signal) in signals_json.as_object().unwrap().iter() {
        let mut signals_item = SignalItem::new(id.parse().unwrap());
        for (name, value) in signal.as_object().unwrap().iter() {
            let signal = Signal::new(
                value["name"].as_str().unwrap().to_string(),
                value["label"].as_str().unwrap().to_string(),
                value["startBit"].as_i64().unwrap(),
                value["bitLength"].as_i64().unwrap(),
                value["factor"].as_f64().unwrap(),
                value["offset"].as_i64().unwrap(),
                value["min"].as_i64().unwrap(),
                value["max"].as_f64().unwrap(),
                value["sourceUnit"].as_str().unwrap().to_string(),
                value["dataType"].as_str().unwrap().to_string(),
                value["interval"].as_i64().unwrap(),
                value["category"].as_str().unwrap().to_string(),
                value["lineInDbc"].as_i64().unwrap(),
                value["states"].as_str().unwrap().to_string(),
                value["msgId"].as_u64().unwrap(),
                value["msgName"].as_str().unwrap().to_string(),
            );
            signals_item.add_signal(signal);
        }
        signals.push(signals_item);
    }
    signals
}

pub fn create_index(json: &str) -> Vec<SignalsIndexItem> {
    let mut signals_index: Vec<SignalsIndexItem> = Vec::new();
    let signals_json: serde_json::Value = serde_json::from_str(json).expect("Failed to parse JSON");
    for (id, signal) in signals_json.as_object().unwrap().iter() {
        let signals_index_item = SignalsIndexItem::new(
            signal["name"].as_str().unwrap().to_string(),
            id.parse().unwrap(),
        );
        signals_index.push(signals_index_item);
    }
    signals_index
}
