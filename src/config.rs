use proxify::common::utils::{validate_ip_address, validate_port};
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::verbose_print::VerbosityLevel;
use crate::VERBOSITY;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1";
const DEFAULT_BIND_PORT: u16 = 65432_u16;
const DEFAULT_NR_PROXIES: u8 = 20_u8;
const MIN_NR_PROXIES: u8 = 2_u8;
const MAX_NR_PROXIES: u8 = 50_u8;

pub struct ProxifyConfig {
    pub bind_addr: String,
    pub bind_port: u16,
    pub nr_of_proxies: u8,
}

impl<'a> ProxifyConfig {
    pub fn new(config_string: &'a String) -> Result<Self, &'static str> {
        Self::parse_config(config_string)
    }

    fn parse_keyvals(config_str: &'a String) -> Vec<(&'a str, &'a str)> {
        let mut res: Vec<(&'a str, &'a str)> = Vec::new();

        for setting in config_str.split(';') {
            Spam!("Found setting '{}'", setting);
            let (key, val) = setting.split_once('=').unwrap_or(("", ""));
            if key.is_empty() || val.is_empty() {
                Spam!("Setting key or value is empty, skipping {}", setting);
                continue;
            }
            Spam!("Adding {} = {}", key, val);
            res.push((key, val));
        }
        res
    }

    fn get_value_from_key(keyvals: &Vec<(&'a str, &'a str)>, key: &'static str) -> Option<&'a str> {
        for (k, v) in keyvals {
            if *k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn parse_config(config_str: &'a String) -> Result<ProxifyConfig, &'static str> {
        let pairs = Self::parse_keyvals(config_str);
        let bind_addr = Self::get_value_from_key(&pairs, "bind_addr")
            .unwrap_or(DEFAULT_BIND_ADDR)
            .to_string();
        let bind_port = match Self::get_value_from_key(&pairs, "bind_port") {
            Some(v) => v.to_string().trim().parse::<u16>().unwrap_or(0),
            None => DEFAULT_BIND_PORT
        };
        let nr_of_proxies = match Self::get_value_from_key(&pairs, "nr_proxies") {
            Some(v) => match v.to_string().trim().parse::<u8>() {
                Ok(v) if v < MIN_NR_PROXIES => 0,
                Ok(v) => v,
                Err(_) => 0
            }
            None => DEFAULT_NR_PROXIES
        };

        if !validate_ip_address(&bind_addr) {
            return Err("Invalid IP address");
        }

        if !validate_port(bind_port) {
            return Err("Invalid port specified");
        }

        if nr_of_proxies < 1 || nr_of_proxies > MAX_NR_PROXIES {
            return Err("Invalid nr_of_proxies");
        }

        Ok(ProxifyConfig {
            bind_addr: bind_addr,
            bind_port: bind_port,
            nr_of_proxies: nr_of_proxies,
        })
    }
}
