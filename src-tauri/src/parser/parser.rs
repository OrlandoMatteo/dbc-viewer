use crate::can::messages::Message;
use crate::can::signals::Signal;

use crate::parser::utils::extract_signal_data;
use crate::parser::utils::extract_signal_id;
use crate::parser::utils::extract_val_data;
use crate::parser::utils::split_can_id;

pub fn parse_dbc(dbc_string: &String) -> (Vec<Message>, Vec<Signal>) {
    // Split the DBC string into lines
    let dbc_lines: Vec<&str> = dbc_string.split("\n").collect();
    let mut messages: Vec<Message> = Vec::new();
    let mut signals: Vec<Signal> = Vec::new();
    // Parse each line into tokens, handling quoted strings
    let _dbc_data: Vec<Vec<&str>> = Vec::new();
    let mut counter: usize = 0;
    let mut current_message: Message = Message::new();
    for line in dbc_lines {
        let mut tokens = Vec::new();
        let _current_token = String::new();
        let _in_quote = false;
        tokens = line.split_whitespace().collect();
        // Data structures for storing parsed information
        //let mut val_list: Vec<T> = Vec::new();
        //let mut data_type_list = Vec::new();
        //let mut comment_list = Vec::new();
        //let mut signal_id_list = Vec::new();
        let problems = Vec::new();

        // Process each parsed line
        if tokens.is_empty() {
            continue; // Skip empty lines
        }

        match tokens[0] {
            "BO_" => {
                if tokens.len() != 5 {
                    //Error
                    break;
                }
                let can_id_str = tokens[1];
                let mut name = String::from(tokens[2]);
                name.truncate(name.len() - 1);
                let dlc_str = tokens[3];
                let can_id: i64 = can_id_str.parse::<i64>().unwrap() & 0x1fffffff;
                // Parse DLC
                let dlc = dlc_str.parse::<u16>().unwrap();
                // Split CAN ID (optional)
                let mut mex: Message = Message::new();
                match split_can_id(can_id) {
                    Ok((is_extended_frame, priority, pgn, source)) => {
                        mex.can_id = can_id;
                        mex.pgn = pgn;
                        mex.source = source;
                        mex.priority = priority;
                        mex.is_extended_frame = is_extended_frame;
                        mex.dlc = dlc;
                        mex.name = name.clone();
                        mex.line_in_dbc = counter as i64;
                        mex.problems = problems.clone();
                    }
                    Err(_err) => continue,
                };
                if current_message.can_id != 0 {
                    //   println!("Error: no signals found for the current message");
                    messages.push(current_message);
                    current_message = mex.clone();
                } else {
                    current_message = mex.clone();
                }
            }
            "SG_" => {
                match extract_signal_data(
                    line,
                    current_message.label.clone(),
                    current_message.name.clone(),
                    counter,
                    current_message.can_id.clone(),
                ) {
                    Ok(signal) => {
                        signals.push(signal.clone());
                        current_message.signals.push(signal.name);
                    }
                    Err(_err) => continue,
                }
                // Handle SG_ lines
                // ... (parse SG_ line details)
            }
            "VAL_" => {
                match extract_val_data(line) {
                    Ok(states) => {
                        for s in &mut signals {
                            if tokens[2] == s.name {
                                s.states = states.clone();
                            }
                        }
                    }
                    Err(_err) => continue,
                }
                // Handle VAL_ lines
                // ... (parse VAL_ line details)
            }
            "BA_" => {
                if tokens.len() == 6 {
                    if tokens[1].contains("CI_SigId") {
                        match extract_signal_id(&tokens) {
                            Ok(sig_id) => {
                                for s in &mut signals {
                                    if tokens[4] == s.name {
                                        s.sig_id = sig_id.clone();
                                    }
                                }
                            }
                            Err(_err) => continue,
                        }
                    }
                }
            }
            _ => {

                // Handle unknown lines (optionally report a warning)
            }
        }
        counter += 1;
    }
    (messages, signals)
}
