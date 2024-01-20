use proxify::common::utils;
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::verbose_print::VerbosityLevel;
use crate::VERBOSITY;

pub struct ProxifyConfig {
    pub bind_addr: String,
    pub bind_port: u16
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

        let bind_addr = Self::get_value_from_key(&pairs, "bind_addr").unwrap_or("127.0.0.1").to_string();
        let bind_port = Self::get_value_from_key(&pairs, "bind_port").unwrap_or("65432").to_string().trim().parse::<u16>().unwrap();

        Ok(ProxifyConfig {
            bind_addr: bind_addr,
            bind_port: bind_port,
        })
    }
}
