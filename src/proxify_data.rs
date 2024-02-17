pub enum ProxifyCommand {
    REQUEST_GET,
    REQUEST_POST,
    END_SESSION,
}

pub struct ProxifyData {
    session: u8,
    command: ProxifyCommand,
    data: Vec<u8>,
}

impl ProxifyData {
    pub fn unmarshal_bytes(data: Vec<u8>) -> Result<Self, String> {
        Ok(ProxifyData {
            session: 1_u8,
            command: ProxifyCommand::REQUEST_GET,
            data: vec!(1_u8, 2_u8),
        })
    }
}
