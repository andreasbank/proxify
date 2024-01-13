use std::net::{TcpStream};
use std::io::{Read, Write};
use std::sync::{Mutex};
use std::str::from_utf8;
use once_cell::sync::Lazy;

use proxify::common::verbose_print::{VerbosityLevel, Verbosity};
use proxify::{Error, Inform, Detail, Spam};

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
            stream.write(msg).unwrap();
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
