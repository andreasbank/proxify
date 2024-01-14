use curl::easy::Easy;

pub struct ProxyConn {
    curl_handle: Easy,
    prepared: bool,
}

impl ProxyConn {
    pub fn new(self) -> Result<Self, String> {
        Ok(Self {
            curl_handle: Easy::new(),
            prepared: false
        })
    }

    pub fn prepare(self) {
        // TODO: curl::init() in main thread before any use
        // TODO: Add error handling to all unwraps
        /*
        curl_handler.url("https://amazon.se/").unwrap();
        curl_handler.write_function(|data| {
            stdout().write_all(data).unwrap();
            Ok(data.len())
        }).unwrap();
        curl_handler.perform().unwrap();
        */
    }
}
