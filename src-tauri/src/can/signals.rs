use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize, Clone)]
pub struct Problem {
    severity: String,
    line: u32,
    description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct State {
    pub value: i32,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize)]
pub struct Signals {
    signals: Vec<Signal>,
}

pub fn get_card_from_signal(signal: &Signal) -> String {
    // create a card with the signal data
    let card = format!(
        "<div class=\"card\">
    <div class=\"card-body\">
        <h5 class=\"card-title \">{}<span class=\"badge rounded-pill bg-signal float-right\">S</span></h5>
        <h6 class=\"card-subtitle mb-2 text-muted\">{}</h6>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Start bit: {}</div>
        <div class=\"p-2 col bd-highlight\">Bit length: {}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Factor: {}</div>
        <div class=\"p-2 col bd-highlight\">Offset: {}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Min: {}</div>
        <div class=\"p-2 col bd-highlight\">Max: {}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Source unit: {}</div>
        <div class=\"p-2 col bd-highlight\">Signal ID: {}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Interval: {}</div>
        <div class=\"p-2 col bd-highlight\">Category: {}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\"><h3>States</h3>{}</div>
        </div>
        <div class=\"row\">
        <div class=\"p-2 col bd-highlight\">Msg ID: {:#X}</div>
        <div class=\"p-2 col bd-highlight\">Msg Name: {}</div
        </div>
        </div>
    </div>
</div>",
        signal.name,
        signal.label,
        signal.start_bit,
        signal.bit_length,
        signal.factor,
        signal.offset,
        signal.min,
        signal.max,
        signal.source_unit,
        signal.sig_id,
        signal.interval,
        signal.category,
        get_table_for_states(&signal.states),
        signal.msg_id,
        signal.msg_name
    );
    card
}

pub fn get_li_from_signal(signal: &Signal) -> String {
    // create a list item with the signal data
    // the item should have hx-id attribute with the signal name
    // the item should have hx-get attribute with the signal name
    // the item should have hx-target attribute with the id signal_card
    let li = format!(
        //hx-on:click=\"
        //var items = document.querySelectorAll('.list-group-item');
        //// Loop through each element and remove the 'active' class
        //items.forEach(function(item) {{
        //item.classList.remove('active');
        //    }});
        //let newTab = event.target
        //newTab.classList.add('active')\"
        "<li class=\"p-2 list-group-item\" onClick=\"get_signal('{}')\">{}<span class=\"badge rounded-pill bg-signal float-right\">S</span></li> ",
        signal.name, signal.name
    );
    li
}

pub fn get_signals(json: &str) -> Vec<Signal> {
    let data: Signals = serde_json::from_str(&json).unwrap();
    let signals = data.signals;
    println!("Signals: {:?}", signals.len());
    signals
}

// function to search the a vector of SignalItem according to the index of the SignalsIndexItem
// the input is a vector of int with the index of the SignalsIndexItem that match the search
// the function returns a vector of SignalItem that match the search

pub fn search_signals(signals: &Vec<Signal>, query: &str) -> Vec<Signal> {
    let mut result: Vec<Signal> = Vec::new();
    for i in signals.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase().contains(&query.to_lowercase()) {
            result.push(i.clone());
        }
    }
    result
}

// search a signal by its name
pub fn search_signal(signals: &Vec<Signal>, query: &str) -> Option<Signal> {
    for i in signals.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase() == query.to_lowercase() {
            return Some(i.clone());
        }
    }
    None
}

fn get_table_for_states(states: &Vec<State>) -> String {
    let mut table = String::from("<table class=\"table table-hover\">");
    table.push_str("<thead><tr><th>Value</th><th>State</th></tr></thead>");
    table.push_str("<tbody>");
    for state in states.iter() {
        table.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            state.value, state.state
        ));
    }
    table.push_str("</tbody></table>");
    table
}
