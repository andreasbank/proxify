use curl::easy::{Easy, Handler, WriteError};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::fmt;
use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use crate::VERBOSITY;

static CURL_INIT_DONE: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

pub enum ProxyConnProtocol {
    HTTP,
    SOCKS4,
    SOCKS5,
}

#[derive(Debug)]
pub enum ProxyErrorKind {
    ProxyProtocolError,
    ProxyAddressError,
    ProxyPortError,
}

#[derive(Debug)]
pub struct ProxyParseError(ProxyErrorKind);

impl fmt::Display for ProxyParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(
            match self.0 {
                ProxyErrorKind::ProxyProtocolError => "Invalid proxy protocol",
                ProxyErrorKind::ProxyAddressError => "Invalid proxy address",
                ProxyErrorKind::ProxyPortError => "Invalid proxy port",
            }
        )
    }
}

impl FromStr for ProxyConnProtocol {
    type Err = ProxyParseError;

    fn from_str(s: &str) -> Result<Self, ProxyParseError> {
        if s == "http" {
            Ok(ProxyConnProtocol::HTTP)
        } else if s == "socks4" {
            Ok(ProxyConnProtocol::SOCKS4)
        } else if s == "socks5" {
            Ok(ProxyConnProtocol::SOCKS5)
        } else {
            Err(ProxyParseError(ProxyErrorKind::ProxyProtocolError))
        }
    }
}

pub struct ProxyConn {
    id: u16,
    proxy_prot: ProxyConnProtocol,
    proxy_addr: String,
    proxy_port: u16,
    curl_handle: curl::easy::Easy,
    prepared: bool,
}

impl ProxyConn {
    const PREPARE_URL: &'static str = "https://google.com";

    pub fn new(id: u16, prot: ProxyConnProtocol, addr: String, port: u16) -> Self {
        Self::init_curl();

        Self {
            id: id,
            proxy_prot: prot,
            proxy_addr: addr,
            proxy_port: port,
            curl_handle: Easy::new(),
            prepared: false
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub fn init_curl() {
        let mut curl_init_done = CURL_INIT_DONE.lock().unwrap();
        if !*curl_init_done {
            *curl_init_done = true;
            Spam!("Initiating the cURL library");
            curl::init();
        } else {
            Spam!("cURL already initiated");
        }
    }

    pub fn prepare(&mut self) -> Result<bool, String> {
        Spam!("Proxy {} preparing", self.id);

        if let Err(e) = self.curl_handle.url(Self::PREPARE_URL) {
            return Err(format!("Failed to set URL for the cURL handler: {}", e.to_string()));
        }

        self.curl_handle.write_function(|data| {
            Spam!("{:#?}", data);
            Ok(data.len())
        }).unwrap();

        match self.curl_handle.perform() {
            Ok(v) => return Ok(true),
            _ => return Ok(false),
        }
    }
}

