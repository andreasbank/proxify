use curl::easy::{Easy, Handler, List, WriteError};
use once_cell::sync::Lazy;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str;
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
    curl_handle: Easy,
    recv_buf: Arc<Mutex<Vec<u8>>>,
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
            recv_buf: Arc::new(Mutex::new(Vec::new())),
            prepared: false
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub fn is_prepared(&self) -> bool {
        self.prepared
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

        match self.request_get(&Self::PREPARE_URL.to_string(), &None, 5) {
            Ok(_) => Ok(true),
            Err(e) if e.contains("timeout") => {
                println!("Failed!!!!");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }


    pub fn request_get(&mut self, url: &String, headers: &Option<Vec<String>>, timeout_sec: u16) -> Result<Vec<u8>, String> {
        Spam!("Sending request using proxy {}", self.id);

        if let Err(e) = self.curl_handle.url(url) {
            return Err(format!("Failed to set URL {} for the cURL handler: {}",
                               url,
                               e.to_string()));
        }

        /* If headers are set, apply them to the handle */
        if let Some(hdrs) = headers {
            let mut list = List::new();
            for h in hdrs {
                list.append(h).unwrap();
            }
            self.curl_handle.http_headers(list).unwrap();
        }

        /* Set the timeout for the connect operation */
        let mut buf = Vec::new();
        self.curl_handle.connect_timeout(Duration::from_secs(timeout_sec.into())).unwrap();

        /* Set the receiving closure */
        let mut transfer = self.curl_handle.transfer();
        if let Err(e) = transfer.write_function(|recv_data| {
            buf.extend_from_slice(&recv_data);
            Ok(recv_data.len())
        }) {
            return Err(format!("Failed to set write_function: {}", e.to_string()));
        }

        /* Do the request */
        if let Err(e) = transfer.perform() {
            return Err(e.to_string());
        }

        /* We need to drop the transfer to let go of the borrowed buffer */
        drop(transfer);

        self.prepared = true;
        //Spam!("Data received:\n {}", str::from_utf8(&buf).unwrap());

        Ok(buf)
    }

    pub fn request_get_as_string(&mut self, url: &String, headers: &Option<Vec<String>>) -> Result<String, String> {
        match self.request_get(url, headers, 10) {
            Ok(data) => Ok(String::from_utf8_lossy(&data).to_string()),
            Err(e) => Err(e),
        }
    }
}

