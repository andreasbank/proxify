use proxify::common::utils;

pub struct ProxifyConfig {
    prot: String,
    bind_addr: String,
    port: u16
}

impl ProxifyConfig {
    pub fn new(config_string: String) -> Result<Self, &'static str> {
        Ok(ProxifyConfig {
            prot: String::from("tcp"),
            bind_addr: String::from("127.0.0.1"),
            port: 65432
        })
    }

    pub fn parse_listen_config(config: &mut ProxifyConfig) -> Result<(), &'static str> {
        config.prot = String::from("tcp");
        config.bind_addr = String::from("127.0.0.1");
        config.port = 65432;
        Ok(())
    }
}
