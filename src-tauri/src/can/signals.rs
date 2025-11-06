use serde::Deserialize;

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
    format!(r#"
<div class="card">
    <div class="card-body">
        <h5 class="card-title">〰️ {name}</h5>
        <h6 class="card-subtitle mb-2 text-muted">{label}</h6>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Start bit:</span>
                    <span class="text-end">{start_bit}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Bit length:</span>
                    <span class="text-end">{bit_length}</span>
                </div>
            </div>
        </div>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Factor:</span>
                    <span class="text-end">{factor}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Offset:</span>
                    <span class="text-end">{offset}</span>
                </div>
            </div>
        </div>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Min:</span>
                    <span class="text-end">{min}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Max:</span>
                    <span class="text-end">{max}</span>
                </div>
            </div>
        </div>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Source unit:</span>
                    <span class="text-end">{source_unit}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Signal ID:</span>
                    <span class="text-end">{sig_id}</span>
                </div>
            </div>
        </div>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Interval:</span>
                    <span class="text-end">{interval}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Category:</span>
                    <span class="text-end">{category}</span>
                </div>
            </div>
        </div>

        <div class="row g-3 mt-2">
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Msg ID:</span>
                    <span class="text-end">{msg_id:#X}</span>
                </div>
            </div>
            <div class="col-md-6">
                <div class="d-flex justify-content-between border-bottom pb-2">
                    <span class="fw-semibold">Msg Name:</span>
                    <span class="text-end">
                        <a class="link-primary" style="cursor: pointer;" onClick="get_message('{msg_name_esc}')">{msg_name}</a>
                    </span>
                </div>
            </div>
        </div>

        <div class="mt-4">

            {states_table}
        </div>
    </div>
</div>
"#,
            name = signal.name,
            label = signal.label,
            start_bit = signal.start_bit,
            bit_length = signal.bit_length,
            factor = signal.factor,
            offset = signal.offset,
            min = signal.min,
            max = signal.max,
            source_unit = signal.source_unit,
            sig_id = signal.sig_id,
            interval = signal.interval,
            category = signal.category,
            states_table = get_table_for_states(&signal.states),
            msg_id = signal.msg_id,
            msg_name = signal.msg_name,
            msg_name_esc = signal.msg_name.replace("'", "\\'")
    )
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
        "<li class=\"p-2 list-group-item\" onClick=\"get_signal('{}')\">〰️ {}</li> ",
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

pub fn search_signals_by_id(signals: &Vec<Signal>, query: &str) -> Vec<Signal> {
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
    if states.is_empty() {
        return "".to_string()
    }
    let mut table = String::from("<h6 class=\"fw-bold\">States</h6><table class=\"table table-hover\">");
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

pub fn get_details_from_signal(signal: &Signal, accordion_parent: String) -> String {
    let details = format!(
        "<div class=\"accordion-item border-bottom-0\">
            <button class=\"accordion-button collapsed\" type=\"button\" data-bs-toggle=\"collapse\" data-bs-target=\"#{}\" aria-expanded=\"false\" aria-controls=\"{}\">{}</button>
                <div id=\"{}\" class=\"accordion-collapse collapse\" data-bs-parent=\"#{}\">
                    <div class=\"accordion-body border-signal\">
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
                        {}
                        </div>
                        <div class=\"row\">
                        <div class=\"p-2 col bd-highlight\">Msg ID: {:#X}</div>
                        <div class=\"p-2 col bd-highlight\">Msg Name: {}</div>
                        </div>
                    </div>
                </div>
        </div>",
        signal.name,
        signal.name,
        signal.name,
        signal.name,
        accordion_parent,
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
    details
}
