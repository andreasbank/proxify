use std::string::String;
use std::convert::TryFrom;
use std::convert::TryInto;

// TODO: remove logging when all works
use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use crate::VERBOSITY;

pub enum ProxifyCommand {
    REQUEST_GET = 1,
    REQUEST_POST = 2,
    END_SESSION = 3,
}

impl TryFrom<u8> for ProxifyCommand {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == ProxifyCommand::REQUEST_GET as u8 => Ok(ProxifyCommand::REQUEST_GET),
            x if x == ProxifyCommand::REQUEST_POST as u8 => Ok(ProxifyCommand::REQUEST_POST),
            x if x == ProxifyCommand::END_SESSION as u8 => Ok(ProxifyCommand::END_SESSION),
            _ => Err(String::from("Invalid ProxifyCommand")),
        }
    }
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
    pub session: u8,
    pub command: ProxifyCommand,
    pub data: Vec<(ProxifyDataType, u8, Vec<u8>)>,
}

impl ProxifyData {
    pub fn unmarshal_bytes(data: Vec<u8>) -> Result<Self, String> {
        let session = data[0];
        let command: ProxifyCommand = match data[1].try_into() {
            Ok(enum_val) => enum_val,
            Err(e) => return Err(String::from("Invalid ProxifyCommand")),
        };
        let parsed_data = match ProxifyData::parse_tlvs(&data[2..]) {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        Ok(ProxifyData {
            session: session,
            command: command,
            data: parsed_data,
        })
    }

    fn parse_tlvs(data: &[u8]) -> Result<Vec<(ProxifyDataType, u8, Vec<u8>)>, String> {
        let mut tlvs: Vec<(ProxifyDataType, u8, Vec<u8>)> = Vec::new();
        let mut begin = 0;
        let end = data.len();

        loop {
            if begin + 3 > end { break; }

            let tlv_type: ProxifyDataType = match data[begin].try_into() {
                Ok(enum_val) => enum_val,
                Err(_) => return Err(String::from("Invalid u8 for ProdyDataType")),
            };

            let tlv_length: u8 = data[begin + 1];

            if begin + 2 + (tlv_length as usize) > end {
                return Err(format!("Invalid TLV found, not enough data (need {}, found {})",
                                   tlv_length,
                                   end - begin + 3));
            }

            let mut tlv_value: Vec<u8> = Vec::new();

            let slice_begin = begin + 2;
            let slice_end = begin + 2 + tlv_length as usize;
            tlv_value.extend_from_slice(&data[slice_begin..slice_end]);
            tlvs.push((tlv_type, tlv_length, tlv_value));

            // For every loop we move one TLV forward (T, L and D[size])
            begin += 2 + (tlv_length as usize);
        }
        Ok(tlvs)
    }
}
