use crate::can::signals::Signal;
use crate::can::signals::State;
pub fn split_can_id(can_id: u64) -> Result<(bool, u16, u64, u16), String> {
    let is_extended_frame = can_id > 0xffff;

    if is_extended_frame {
        let source = can_id as u16 & 0xff;
        let pgn = (can_id >> 8) as u64 & 0xffff;
        let priority = (can_id >> 24) as u16 & 0xff;
        Ok((is_extended_frame, priority, pgn, source))
    } else {
        Ok((is_extended_frame, 0, can_id, 0))
    }
}

pub fn extract_signal_data(
    _line: &str,
    label_prefix: String,
    message_name: String,
    index: usize,
    can_id: u64,
) -> Result<Signal, String> {
    let line: Vec<&str> = _line.split_whitespace().collect();
    if line.len() < 8 {
        return Err("malformed SG_ signal definition".to_string());
    }

    let name = line
        .get(1)
        .ok_or_else(|| "missing signal name".to_string())?
        .to_string();
    let colon_index = line
        .iter()
        .position(|token| *token == ":")
        .ok_or_else(|| "missing signal separator ':'".to_string())?;

    let bit_token = line
        .get(colon_index + 1)
        .ok_or_else(|| "missing bit layout".to_string())?;
    let factor_token = line
        .get(colon_index + 2)
        .ok_or_else(|| "missing factor/offset".to_string())?;
    let min_max_token = line
        .get(colon_index + 3)
        .ok_or_else(|| "missing min/max".to_string())?;

    let sb_bl_endian: Vec<&str> = bit_token.split('|').collect();
    if sb_bl_endian.len() != 2 {
        return Err(format!("invalid bit layout '{}'", bit_token));
    }
    let start_bit = sb_bl_endian[0]
        .parse::<u32>()
        .map_err(|error| format!("invalid start bit '{}': {}", sb_bl_endian[0], error))?;
    let bl_endian: Vec<&str> = sb_bl_endian[1].split('@').collect();
    if bl_endian.len() != 2 {
        return Err(format!("invalid bit length/endian '{}'", sb_bl_endian[1]));
    }
    let bit_length = bl_endian[0]
        .parse::<u32>()
        .map_err(|error| format!("invalid bit length '{}': {}", bl_endian[0], error))?;
    let little_endian = bl_endian[1].starts_with('1');
    let is_signed = bl_endian[1].contains('-');

    let fac_off = factor_token
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| format!("invalid factor/offset '{}'", factor_token))?;
    let factor_offset: Vec<&str> = fac_off.split(',').collect();
    if factor_offset.len() != 2 {
        return Err(format!("invalid factor/offset '{}'", factor_token));
    }
    let factor = factor_offset[0]
        .parse::<f64>()
        .map_err(|error| format!("invalid factor '{}': {}", factor_offset[0], error))?;
    let offset = factor_offset[1]
        .parse::<f32>()
        .map_err(|error| format!("invalid offset '{}': {}", factor_offset[1], error))?;

    let min_max_value = min_max_token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("invalid min/max '{}'", min_max_token))?;
    let min_max: Vec<&str> = min_max_value.split('|').collect();
    if min_max.len() != 2 {
        return Err(format!("invalid min/max '{}'", min_max_token));
    }
    let min = min_max[0]
        .parse::<f32>()
        .map_err(|error| format!("invalid min '{}': {}", min_max[0], error))?;
    let max = min_max[1]
        .parse::<f32>()
        .map_err(|error| format!("invalid max '{}': {}", min_max[1], error))?;

    let category = line.last().unwrap_or(&"").to_string();
    let source_unit = line
        .get(line.len().saturating_sub(2))
        .unwrap_or(&"")
        .trim_matches('"')
        .to_string();
    let default_string = String::from("");
    Ok(Signal {
        name,
        start_bit,
        bit_length,
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
        is_signed,
        source_unit,
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
    if tokens.len() < 6 {
        return Err("malformed CI_SigId attribute".to_string());
    }

    let sig_id: i32 = tokens[5]
        .replace(";", "")
        .parse::<i32>()
        .map_err(|error| format!("invalid signal ID '{}': {}", tokens[5], error))?;
    Ok(sig_id)
}
