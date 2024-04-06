use std::net::{TcpStream};
use std::io::{Read, Write};
use std::sync::{Mutex};
use std::str::from_utf8;
use once_cell::sync::Lazy;

use proxify::common::verbose_print::{VerbosityLevel, Verbosity};
use proxify::{Error, Inform, Detail, Spam};
use proxify::proxy_conn::ProxyConn;
use proxify::proxy_conn::ProxyConnProtocol;
use proxify::proxify_data::{ProxifyCommand, ProxifyDataType, ProxifyData};

static VERBOSITY: Lazy<Mutex<Verbosity>> = Lazy::new(|| Mutex::new(Verbosity::new()));
static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

fn main() {
    VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Spam);

    match TcpStream::connect("localhost:65432") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 65432");

            let msg = b"Hello!";

            stream.write(&MAGIC_BYTES).unwrap();
            println!("Sent magic bytes");
            // TODO: remove test data
            let fake_data: Vec<u8> = vec!(1_u8,
                                          ProxifyCommand::REQUEST_POST as u8,
                                          ProxifyDataType::URL as u8,
                                          8_u8,
                                          'z' as u8,
                                          'e' as u8,
                                          'l' as u8,
                                          'd' as u8,
                                          'a' as u8,
                                          'b' as u8,
                                          'a' as u8,
                                          'n' as u8);
            stream.write(&fake_data).unwrap();
            println!("Sent Hello, awaiting reply...");

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
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
