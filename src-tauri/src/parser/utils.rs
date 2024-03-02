use crate::can::signals::Signal;
use crate::can::signals::State;
pub fn split_can_id(can_id: i64) -> Result<(bool, u16, u64, u16), String> {
    let is_extended_frame = can_id > 0xffff;
    let mut priority = 0;
    let mut pgn = 0;
    let mut source = 0;

    if is_extended_frame {
        source = can_id as u16 & 0xff;
        pgn = (can_id >> 8) as u64 & 0xffff;
        priority = (can_id >> 24) as u16 & 0xff;
    } else {
        pgn = can_id as u64;
    }

    Ok((is_extended_frame, priority, pgn, source))
}

pub fn extract_signal_data(
    _line: &str,
    label_prefix: String,
    message_name: String,
    index: usize,
    can_id: i64,
) -> Result<Signal, String> {
    // Handle multiplexor field
    //let multi_item = line[2];
    //let (is_multiplexor, multiplexer_value) = match multi_item.len() {
    //    9 if multi_item.chars().nth(3) == Some(':') => {
    //        let raw_multiplexer = multi_item.chars();
    //        if raw_multiplexer.nth(0) == Some('M') {
    //            (true, None)
    //        } else if raw_multiplexer.nth(0).unwrap() == 'm' {
    //            let value: u32 = raw_multiplexer
    //                .skip(1)
    //                .map(|c| c.to_digit(10).unwrap() as u32)
    //                .collect();
    //            (true, Some(value))
    //        } else {
    //            return Err(format!("Error parsing multiplexer: {}", line[2]));
    //        }
    //    }
    //    _ => (false, None),
    //};
    let line: Vec<&str> = _line.split_whitespace().collect();
    let name: String = String::from(line[1].clone());
    // Parse remaining fields
    let sb_bl_endian: Vec<String> = line[3].split('|').map(|s| s.to_string()).collect();
    let mut start_bit: u32 = 0;
    let mut little_endian = false;
    let mut bit_length: u32 = 0;
    if sb_bl_endian.len() > 1 {
        start_bit = sb_bl_endian[0].parse::<u32>().unwrap_or(0);
        let bl_endian: Vec<String> = sb_bl_endian[1].split('@').map(|s| s.to_string()).collect();
        bit_length = bl_endian[0].parse::<u32>().unwrap_or(0);
        little_endian = bl_endian[1].starts_with("1");
    }

    let _fac_off = &line[4][1..line[4].len() - 1];
    let mut factor: f64 = 0.0;
    let mut offset: f32 = 0.0;
    let factor_offset: Vec<String> = _fac_off.split(',').map(|s| s.to_string()).collect();
    if factor_offset.len() > 1 {
        factor = factor_offset[0].parse::<f64>().unwrap_or(0.0);
        offset = factor_offset[1].parse::<f32>().unwrap_or(0.0);
    }

    let _min_max = &line[5][1..line[5].len() - 1];
    let min_max: Vec<String> = _min_max.split('|').map(|s| s.to_string()).collect();
    let mut min = 0.0;
    let mut max = 0.0;
    if min_max.len() > 1 {
        min = min_max[0].parse::<f32>().unwrap_or(0.0);
        max = min_max[1].parse::<f32>().unwrap_or(0.0);
    }

    // Categorize signal
    let category = String::from(line[line.len() - 1]);
    let source_unit = String::from(line[line.len() - 2]);
    let default_string = String::from("");
    Ok(Signal {
        name: name,
        start_bit: start_bit,
        bit_length: bit_length,
        is_little_endian: little_endian,
        factor,
        offset,
        min,
        max,
        visibility: true, // ViriCiti specific
        interval: 1000,   // ViriCiti specific
        category: category,
        line_in_dbc: index as u32,
        label: label_prefix,
        is_signed: false,
        source_unit: source_unit,
        data_type: default_string.clone(),
        choking: false,
        problems: Vec::new(),
        postfix_metric: default_string.clone(),
        states: Vec::new(),
        msg_id: can_id as u64,
        msg_name: message_name,
        sig_id: 0,
    })
}

pub fn extract_val_data(_line: &str) -> Result<Vec<State>, String> {
    let mut states: Vec<State> = Vec::new();
    let joined = _line
        .split_whitespace()
        .skip(3)
        .collect::<Vec<&str>>()
        .join(" ");

    let descriptions: Vec<&str> = joined.split('"').collect();
    let number_of_states = descriptions.len() / 2;
    for i in 0..number_of_states {
        let value = descriptions[i * 2]
            .trim()
            .parse::<i32>()
            .unwrap_or_default();
        let state = String::from(descriptions[i * 2 + 1]);
        states.push(State { value, state });
    }

    Ok(states)
}

pub fn extract_signal_id(tokens: &Vec<&str>) -> Result<i32, String> {
    let sig_id: i32 = tokens[5]
        .replace(";", "")
        .parse::<i32>()
        .unwrap_or_default();
    Ok(sig_id)
}
