use curl::easy::Easy;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use crate::VERBOSITY;

static CURL_INIT_DONE: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

pub enum ProxyConnProtocol {
    HTTP,
    SOCKS4,
    SOCKS5,
}

pub struct ProxyConn {
    proxy_prot: ProxyConnProtocol,
    proxy_addr: String,
    proxy_port: u16,
    curl_handle: Easy,
    prepared: bool,
}

impl ProxyConn {
    const PREPARE_URL: &'static str = "https://google.com";

    pub fn new(prot: ProxyConnProtocol, addr: String, port: u16) -> Self {
        Self::init_curl();

        Self {
            proxy_prot: prot,
            proxy_addr: addr,
            proxy_port: port,
            curl_handle: Easy::new(),
            prepared: false
        }
    }

    pub fn init_curl() {
        let mut curl_init_done = CURL_INIT_DONE.lock().unwrap();
        if !*curl_init_done {
            *curl_init_done = true;
            Spam!("Initiating the cURL library");
            curl::init();
        }
        Spam!("cURL already initiated");
    }

    pub fn prepare(&self) {
        /*
        // TODO: Add error handling to all unwraps
        curl_handler.url("https://amazon.se/").unwrap();
        curl_handler.write_function(|data| {
            stdout().write_all(data).unwrap();
            Ok(data.len())
        }).unwrap();
        curl_handler.perform().unwrap();
        */
    }
}
