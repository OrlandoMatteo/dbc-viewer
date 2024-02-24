use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    #[serde(default = "default_u64")]
    can_id: u64,
    pgn: u64,
    source: u16,
    name: String,
    priority: u16,
    label: String,
    #[serde(rename = "isExtendedFrame")]
    is_extended_frame: bool,
    dlc: u16,
    comment: Option<String>,
    #[serde(default = "default_u64")]
    line_in_dbc: u64,
    problems: Vec<String>,
    pub signals: Vec<String>,
}
fn default_u64() -> u64 {
    0
}

#[derive(Debug, Deserialize)]
pub struct Messages {
    messages: Vec<Message>,
}

pub fn get_messages(json: &str) -> Vec<Message> {
    let data: Messages = serde_json::from_str(&json).unwrap();
    let messages = data.messages;
    println!("messages: {:?}", messages.len());
    messages
}

pub fn search_messages_by_name(messages: &Vec<Message>, query: &str) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    for i in messages.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase().contains(&query.to_lowercase()) {
            result.push(i.clone());
        }
    }
    result
}
pub fn search_messages_by_id(messages: &Vec<Message>, query: &str) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    for i in messages.iter() {
        // th
        // if the id of the signal contains the query, ignore case
        if i.can_id == query.parse::<u64>().unwrap() {
            result.push(i.clone());
        }
    }
    result
}
pub fn search_messages_by_signal(messages: &Vec<Message>, query: &str) -> Vec<Message> {
    let mut result: Vec<Message> = Vec::new();
    for i in messages.iter() {
        // th
        // check all the signals in the message
        for s in i.signals.iter() {
            // if the name of the signal contains the query, ignore case
            if s.to_lowercase().contains(&query.to_lowercase()) {
                result.push(i.clone());
            }
        }
    }
    result
}

pub fn get_li_from_message(message: &Message) -> String {
    // create a list item with the message data
    // the item should have hx-id attribute with the message name
    // the item should have hx-get attribute with the message name
    // the item should have hx-target attribute with the id signal_card
    let query_value = json!({"query":message.name});
    let li = format!(
        "<li 
        hx-on:click=\"
        var items = document.querySelectorAll('.list-group-item');
        // Loop through each element and remove the 'active' class
        items.forEach(function(item) {{
        item.classList.remove('active');
            }});
        let newTab = event.target
        newTab.classList.add('active')\"
            class=\"p-2 list-group-item\" hx-post=\"command:show_message\" name=query hx-vals={} hx-target=\"#signal_card\" hx-swap=innerHTML  >{}</li>
        ",
        query_value, message.name
    );
    li
}

pub fn get_card_from_message(message: &Message) -> String {
    // create a card with the signal data

    let mut signal_str = String::from("<ul class=\"list-group\">");
    for signal in &message.signals {
        let sig_li = format!("<li class=\"list-group-item\">{}</li>", signal);
        signal_str.push_str(&sig_li)
    }

    let card = format!(
        "<div class=\"card\">
    <div class=\"card-body\">
        <h5 class=\"card-title\">{}</h5>
        <div class=\"bd-highlight mb-3\">
        <div class=\"p-2 bd-highlight\">CAN ID: {}</div>
        <div class=\"p-2 bd-highlight\">PGN: {}</div>
        <div class=\"p-2 bd-highlight\">Signals: {}</div
        </div>
    </div>
</div>",
        message.name, message.can_id, message.pgn, signal_str
    );
    card
}
pub fn search_message(messages: &Vec<Message>, query: &str) -> Option<Message> {
    for i in messages.iter() {
        // if the name of the signal contains the query, ignore case
        if i.name.to_lowercase() == query.to_lowercase() {
            return Some(i.clone());
        }
    }
    None
}
