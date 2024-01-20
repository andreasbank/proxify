use std::{fmt::Write as FmtWrite, num::ParseIntError};
use std::net::IpAddr;
use std::str::FromStr;

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

pub fn validate_port(port: u16) -> bool{
    if port < 100 {
        return false;
    }
    true
}

pub fn validate_ip_address(ip_addr: &String) -> bool {
    match IpAddr::from_str(ip_addr.as_str()) {
        Ok(_) => return true,
        Err(_) => return false
    }
}
