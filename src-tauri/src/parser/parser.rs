use crate::can::messages::Message;
use crate::can::signals::Signal;

use crate::parser::utils::extract_signal_data;
use crate::parser::utils::extract_signal_id;
use crate::parser::utils::extract_val_data;
use crate::parser::utils::split_can_id;

#[derive(Debug)]
pub struct ParseResult {
    pub messages: Vec<Message>,
    pub signals: Vec<Signal>,
    pub warnings: Vec<String>,
}

pub fn parse_dbc(dbc_string: &str) -> ParseResult {
    let mut messages: Vec<Message> = Vec::new();
    let mut signals: Vec<Signal> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut current_message: Option<Message> = None;

    for (line_index, line) in dbc_string.lines().enumerate() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "BO_" => {
                if tokens.len() != 5 {
                    warnings.push(format!(
                        "Line {}: malformed BO_ message definition",
                        line_index + 1
                    ));
                    continue;
                }

                let can_id = match tokens[1].parse::<u64>() {
                    Ok(can_id) => can_id & 0x1fffffff,
                    Err(error) => {
                        warnings.push(format!(
                            "Line {}: invalid CAN ID '{}': {}",
                            line_index + 1,
                            tokens[1],
                            error
                        ));
                        continue;
                    }
                };

                let dlc = match tokens[3].parse::<u16>() {
                    Ok(dlc) => dlc,
                    Err(error) => {
                        warnings.push(format!(
                            "Line {}: invalid DLC '{}': {}",
                            line_index + 1,
                            tokens[3],
                            error
                        ));
                        continue;
                    }
                };

                if let Some(message) = current_message.take() {
                    messages.push(message);
                }

                let mut name = tokens[2].to_string();
                if name.ends_with(':') {
                    name.pop();
                }

                let mut message = Message::new();
                match split_can_id(can_id) {
                    Ok((is_extended_frame, priority, pgn, source)) => {
                        message.can_id = can_id;
                        message.pgn = pgn;
                        message.source = source;
                        message.priority = priority;
                        message.is_extended_frame = is_extended_frame;
                        message.dlc = dlc;
                        message.name = name;
                        message.line_in_dbc = line_index as i64;
                    }
                    Err(error) => {
                        warnings.push(format!("Line {}: {}", line_index + 1, error));
                        continue;
                    }
                };

                current_message = Some(message);
            }
            "SG_" => {
                let Some(message) = current_message.as_mut() else {
                    warnings.push(format!(
                        "Line {}: signal declared before any message",
                        line_index + 1
                    ));
                    continue;
                };

                match extract_signal_data(
                    line,
                    message.label.clone(),
                    message.name.clone(),
                    line_index,
                    message.can_id,
                ) {
                    Ok(signal) => {
                        message.signals.push(signal.name.clone());
                        signals.push(signal);
                    }
                    Err(error) => warnings.push(format!("Line {}: {}", line_index + 1, error)),
                }
            }
            "VAL_" => {
                if tokens.len() < 4 {
                    warnings.push(format!(
                        "Line {}: malformed VAL_ state definition",
                        line_index + 1
                    ));
                    continue;
                }

                match extract_val_data(line) {
                    Ok(states) => {
                        let signal_name = tokens[2];
                        if let Some(signal) =
                            signals.iter_mut().find(|signal| signal.name == signal_name)
                        {
                            signal.states = states;
                        } else {
                            warnings.push(format!(
                                "Line {}: states reference unknown signal '{}'",
                                line_index + 1,
                                signal_name
                            ));
                        }
                    }
                    Err(error) => warnings.push(format!("Line {}: {}", line_index + 1, error)),
                }
            }
            "BA_" => {
                if tokens.len() == 6 && tokens[1].contains("CI_SigId") {
                    match extract_signal_id(&tokens) {
                        Ok(sig_id) => {
                            let signal_name = tokens[4];
                            if let Some(signal) =
                                signals.iter_mut().find(|signal| signal.name == signal_name)
                            {
                                signal.sig_id = sig_id;
                            } else {
                                warnings.push(format!(
                                    "Line {}: signal ID references unknown signal '{}'",
                                    line_index + 1,
                                    signal_name
                                ));
                            }
                        }
                        Err(error) => warnings.push(format!("Line {}: {}", line_index + 1, error)),
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(message) = current_message {
        messages.push(message);
    }

    ParseResult {
        messages,
        signals,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_dbc;

    const SAMPLE_DBC: &str = r#"
BO_ 256 EngineData: 8 Vector__XXX
 SG_ VehicleSpeed : 0|16@1+ (0.1,0) [0|250] "km/h" Vector__XXX
VAL_ 256 VehicleSpeed 0 "Stopped" 1 "Moving" ;
BA_ "CI_SigId" SG_ 256 VehicleSpeed 1234;
"#;

    #[test]
    fn parses_one_message_and_keeps_final_message() {
        let parsed = parse_dbc(SAMPLE_DBC);

        assert_eq!(parsed.messages.len(), 1);
        assert_eq!(parsed.messages[0].name, "EngineData");
        assert_eq!(parsed.messages[0].signals, vec!["VehicleSpeed"]);
        assert_eq!(parsed.signals.len(), 1);
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn parses_states_and_signal_id() {
        let parsed = parse_dbc(SAMPLE_DBC);
        let signal = &parsed.signals[0];

        assert_eq!(signal.sig_id, 1234);
        assert_eq!(signal.states.len(), 2);
        assert_eq!(signal.states[0].value, 0);
        assert_eq!(signal.states[0].state, "Stopped");
        assert_eq!(signal.states[1].value, 1);
        assert_eq!(signal.states[1].state, "Moving");
    }

    #[test]
    fn malformed_lines_return_warnings_without_panicking() {
        let parsed = parse_dbc(
            r#"
BO_ not-a-valid-message
SG_ MissingBits : nope
VAL_ 1 UnknownSignal 0 "Nope" ;
"#,
        );

        assert!(parsed.messages.is_empty());
        assert!(parsed.signals.is_empty());
        assert!(!parsed.warnings.is_empty());
    }
}
