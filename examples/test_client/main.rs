use std::net::{TcpStream};
use std::io::{Read, Write};
use std::string;
use std::sync::{Mutex};
use std::str::from_utf8;
use once_cell::sync::Lazy;

use proxify::common::VERBOSITY;
use proxify::common::verbose_print::{VerbosityLevel, Verbosity};
use proxify::{Error, Warn, Inform, Detail, Spam};
use proxify::proxy_conn::ProxyConn;
use proxify::proxy_conn::ProxyConnProtocol;
use proxify::proxify_data::{ProxifyCommand, ProxifyDataType, ProxifyData};

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

fn main() {
    VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Spam);
    Spam!("Log level spam is active");

    match TcpStream::connect("localhost:65432") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 65432");

            stream.write(&MAGIC_BYTES).unwrap();
            println!("Sent magic bytes");

            let mut data = [0 as u8; 4]; // using 4 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == MAGIC_BYTES {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }

            /* Ask the proxify daemon to make a request to http://google.com */
            let mut send_data = String::from("http://google.com").into_bytes();
            let mut test_command: Vec<u8> = vec!(1_u8,
                                                 ProxifyCommand::REQUEST_POST as u8,
                                                 ProxifyDataType::URL as u8,
                                                 send_data.len() as u8);
            test_command.append(&mut send_data);
            stream.write(&test_command).unwrap();
            println!("Sent data, awaiting reply...");

            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data == MAGIC_BYTES {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }

        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
