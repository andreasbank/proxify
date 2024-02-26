use std::string::String;
use std::convert::TryFrom;
use std::convert::TryInto;

pub enum ProxifyCommand {
    REQUEST_GET,
    REQUEST_POST,
    END_SESSION,
}

pub enum ProxifyDataType {
    URL = 1,
    HEADER = 2,
    DATA = 3,
}

/* Read more on the code below:
   https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
*/
impl TryFrom<u8> for ProxifyDataType {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == ProxifyDataType::URL as u8 => Ok(ProxifyDataType::URL),
            x if x == ProxifyDataType::HEADER as u8 => Ok(ProxifyDataType::HEADER),
            x if x == ProxifyDataType::DATA as u8 => Ok(ProxifyDataType::DATA),
            _ => Err(String::from("Invalid ProxifyDataType")),
        }
    }
}

pub struct ProxifyData {
    session: u8,
    command: ProxifyCommand,
    data: Vec<(ProxifyDataType, u8, Vec<u8>)>,
}

impl ProxifyData {
    pub fn unmarshal_bytes(data: Vec<u8>) -> Result<Self, String> {
        // TODO: Finish below
        let parsed_data = match ProxifyData::parse_tlvs(data) {
            Ok(pd) => pd,
            Err(e) => return Err(e),
        };

        // TODO fix this when all tlv parsing works
        Ok(ProxifyData {
            session: 1_u8,
            command: ProxifyCommand::REQUEST_GET,
            data: vec!((ProxifyDataType::DATA, 1_u8, vec!(2_u8))),
        })
    }

    fn parse_tlvs(data: Vec<u8>) -> Result<Vec<(ProxifyDataType, u8, Vec<u8>)>, String> {
        let mut tlvs: Vec<(ProxifyDataType, u8, Vec<u8>)> = Vec::new();
        let mut begin = 0;
        let end = data.len();

        loop {
            if begin + 3 < end { break; }

            let tlv_type: ProxifyDataType = match data[begin].try_into() {
                Ok(ProxifyDataType::URL) => ProxifyDataType::URL,
                Ok(ProxifyDataType::HEADER) => ProxifyDataType::HEADER,
                Ok(ProxifyDataType::DATA) => ProxifyDataType::DATA,
                Err(_) => return Err(String::from("Invalid u8 for ProdyDataType")),
            };

            let tlv_length: u8 = data[begin + 1];

            if begin + 3  + (tlv_length as usize) < end {
                return Err(format!("Invalid TLV found, not enough data (need {}, found {})",
                                   tlv_length,
                                   end - begin + 3));
            }

            let mut tlv_value: Vec<u8> = Vec::new();

            tlv_value.extend_from_slice(&data[(begin + 2)..(tlv_length as usize)]);
            tlvs.push((tlv_type, tlv_length, tlv_value));

            // For every loop we move one TLV forward (T, L and D[size])
            begin += 2 + (tlv_length as usize);
        }
        Ok(tlvs)
    }
}
