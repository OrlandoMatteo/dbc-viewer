use crate::can::messages::Message;
use crate::can::signals::Signal;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileUpload {
    pub file: String, // Change the type to match your file data type
}

pub fn parse_dbc(dbc: &str) -> Vec<Message> {
    println!("{}", dbc);
    let output: Vec<Message> = Vec::new();
    return output;
}
