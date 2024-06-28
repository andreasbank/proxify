use std::fs::read_to_string;
use crate::common::utils::{validate_ip_address, validate_port};
use crate::common::VERBOSITY;
use crate::{Error, Warn, Inform, Detail, Spam};
use crate::common::verbose_print::VerbosityLevel;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1";
const DEFAULT_BIND_PORT: u16 = 65432_u16;
const DEFAULT_NR_PROXIES: u8 = 20_u8;
const DEFAULT_NR_PREPARE_THREADS: u8 = 1_u8;
const MIN_NR_PROXIES: u8 = 2_u8;
const MAX_NR_PROXIES: u8 = 50_u8;
const MAX_NR_PREPARE_THREADS: u8 = 50_u8;

pub struct ProxifyConfig {
    pub bind_addr: String,
    pub bind_port: u16,
    pub nr_of_proxies: u8,
    pub nr_of_prepare_threads: u8,
    pub proxies_list: Vec<(String, String, u16, Option<String>, Option<String>)>,
}

impl<'a> ProxifyConfig {
    pub fn new(config_string: &'a String) -> Result<Self, String> {
        Self::parse_config(config_string)
    }

    fn parse_keyvals(config_str: &'a String) -> Vec<(&'a str, &'a str)> {
        let mut res: Vec<(&'a str, &'a str)> = Vec::new();

        for setting in config_str.split(';') {
            Spam!("Found setting '{}'", setting);
            let (key, val) = setting.split_once('=').unwrap_or(("", ""));
            if key.is_empty() || val.is_empty() {
                Warn!("Setting key or value is empty, skipping {}", setting);
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

    pub fn parse_config(config_str: &'a String) -> Result<ProxifyConfig, String> {
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

        let proxies_file = Self::get_value_from_key(&pairs, "proxies_file")
            .unwrap_or("proxies.json")
            .to_string();

        let nr_of_prepare_threads = match Self::get_value_from_key(&pairs, "nr_prepare_threads") {
            Some(v) => match v.to_string().trim().parse::<u8>() {
                Ok(v) => v,
                Err(_) => 0
            }
            None => DEFAULT_NR_PREPARE_THREADS
        };

        if !validate_ip_address(&bind_addr) {
            return Err(String::from("Invalid IP address"));
        }

        if !validate_port(bind_port) {
            return Err(String::from("Invalid port specified"));
        }

        if nr_of_proxies < 1 || nr_of_proxies > MAX_NR_PROXIES {
            return Err(String::from("Invalid nr_proxies"));
        }

        if nr_of_prepare_threads > MAX_NR_PREPARE_THREADS {
            return Err(String::from("Invalid nr_prepare_threads"));
        }

        let proxies_list = match Self::parse_proxies_file(&proxies_file) {
            Ok(list) => list,
            Err(e) => return Err(format!("Failed to parse proxies file ({}): {}",
                                 proxies_file,
                                 e.to_string())),
        };

        Ok(ProxifyConfig {
            bind_addr: bind_addr,
            bind_port: bind_port,
            nr_of_proxies: nr_of_proxies,
            nr_of_prepare_threads: nr_of_prepare_threads,
            proxies_list: proxies_list,
        })
    }

    fn parse_proxies_file(proxies_file: &String) ->
        Result<Vec<(String, String, u16, Option<String>, Option<String>)>, String> {
        let lines_string: String = match read_to_string(proxies_file) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to read file '{}': {}", proxies_file, e)),
        };
        let lines: Vec<String> = lines_string.lines().map(String::from).collect();
        let mut proxies: Vec<(String, String, u16, Option<String>, Option<String>)> = Vec::new();

        /* Example for a proxy url: "http://url.com:3128" */
        for line in lines {
            /* Split "http" and "url.com:3128" */
            let (prot, uname_pass_url_port) = match line.split_once("://") {
                Some((p, up)) => (p.to_string(), up),
                None => return Err(format!("Failed to parse proxy protocol from '{}'", line)),
            };

            /* Split "username[:password]@" and "url.com:3128" */
            //TODO: ovaj
            let (uname_pass, url_port) = match uname_pass_url_port.split_once('@') {
                Some((unamepass, urlport)) => (Some(unamepass), urlport),
                None => (None, uname_pass_url_port)
            };

            /* Split "username" and "password" */
            let mut uname: Option<String> = None;
            let mut pass: Option<String> = None;
            if let Some(unamepass) = uname_pass {
                (uname, pass) = match unamepass.split_once(':') {
                    Some((u, p)) => (Some(u.to_string()), Some(p.to_string())),
                    None => (Some(unamepass.to_string()), None)
                };
            }

            /* Split "url.com" and "3128" */
            let (url, port) = match url_port.split_once(':') {
                Some((u, p)) => (u.to_string(), match p.parse::<u16>() {
                                                    Ok(v) => v,
                                                    Err(_) => return Err(format!("Failed to parse proxy port from '{}'", url_port)),
                                                }),
                None => return Err(format!("Failed to parse URL and port from '{}'", url_port)),
            };
            Spam!("Parsed proxy: '{}', '{}', '{}', '{}', '{}'",
                  &prot,
                  &url,
                  &port,
                  uname.clone().unwrap_or(String::from("(none)")),
                  pass.clone().unwrap_or(String::from("(none)")));
            proxies.push((prot, url, port, uname, pass));
        }
        Ok(proxies)
    }
}
