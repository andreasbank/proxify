use std::string::String;

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

/*
TODO: Implement the following:
(https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer)

use std::convert::TryFrom;

impl TryFrom<i32> for MyEnum {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == MyEnum::A as i32 => Ok(MyEnum::A),
            x if x == MyEnum::B as i32 => Ok(MyEnum::B),
            x if x == MyEnum::C as i32 => Ok(MyEnum::C),
            _ => Err(()),
        }
    }
}

use std::convert::TryInto;

fn main() {
    let x = MyEnum::C as i32;

    match x.try_into() {
        Ok(MyEnum::A) => println!("a"),
        Ok(MyEnum::B) => println!("b"),
        Ok(MyEnum::C) => println!("c"),
        Err(_) => eprintln!("unknown number"),
    }
}}
*/

pub struct ProxifyData {
    session: u8,
    command: ProxifyCommand,
    data: Vec<u8>,
}

impl ProxifyData {
    pub fn unmarshal_bytes(data: Vec<u8>) -> Result<Self, String> {
        // TODO Then fix this!
        Ok(ProxifyData {
            session: 1_u8,
            command: ProxifyCommand::REQUEST_GET,
            data: vec!(1_u8, 2_u8),
        })
    }

    // TODO: change return to Result<Vec<(ProxifyDataType, u8, Vec<u8>)>, String>
    //       when the conversion works
    fn parse_tlvs(data: Vec<u8>) -> Result<Vec<(u8, u8, Vec<u8>)>, String> {
        let mut tlvs: Vec<(u8, u8, Vec<u8>)> = Vec::new();

        let mut begin = 0;
        let end = data.len();
        loop {
            if begin + 3 < end { break; }

            let tlv_type: u8 = data[0]; // <----- Convert to ProxifyDataType
            let tlv_length: u8 = data[1];
            let mut tlv_value: Vec<u8> = Vec::new();

            for value in data.clone().into_iter().skip(2) {
                tlv_value.push(value);
            }

            tlvs.push((tlv_type, tlv_length, tlv_value));
            begin += 3;
        }
        Ok(tlvs)
    }
}
