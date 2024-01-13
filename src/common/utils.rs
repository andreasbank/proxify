use std::{fmt::Write as FmtWrite, num::ParseIntError};

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut hex, "{:02X}", b).unwrap();
    }
    hex
}

pub fn parse_address(config_str: &String) -> String {
    let addr = match config_str.rfind("addr=") {
        Some(a) => &config_str[a..config_str.find(",").unwrap()],
        None => "0.0.0.0"
    };

    String::from(addr)
}
