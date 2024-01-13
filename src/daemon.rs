use std::thread;
use std::string::String;
use std::result::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, Shutdown};

use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::utils::encode_hex;
use crate::VERBOSITY;

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

pub struct ProxifyDaemon {
    addr: String,
    port: u16,
    listener: Option<TcpListener>
}

impl ProxifyDaemon {
    pub fn new(addr: &String, port: u16) -> Result<Self, &'static str> {
        if port < 100 {
            return Err("Invalid port");
        }
        // TODO: also validate address

        Ok(ProxifyDaemon {
            addr: addr.clone(),
            port: port,
            listener: None
        })
    }

    pub fn start(&mut self) -> std::io::Result<()>{
        self.listener = Some(TcpListener::bind((self.addr.as_str(), self.port)).unwrap());

        for stream in self.listener.as_ref().unwrap().incoming() {
            match stream {
                Ok(stream) => {
                    Inform!("Accepted connection from address {}", stream.peer_addr().unwrap());
                    thread::spawn(move|| {
                        Self::handle_accept(stream)
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        // Somehow stop self.listener;
    }

    fn authenticate(data: &[u8]) -> Result<(), &'static str> {
        Spam!("Magic bytes received: {}", encode_hex(data));
        if MAGIC_BYTES == data { return Ok(()); }

        Err("Wrong magic bytes")
    }

    fn handle_accept(mut stream: TcpStream) {
        let mut authenticated = false;
        let mut data = [0 as u8; 1024];

        loop {
            match stream.read(&mut data) {

                Ok(size) if size > 0 => {

                    /* Check for the magic bytes */
                    if !authenticated {
                        match Self::authenticate(&data[0..MAGIC_BYTES.len()]) {
                            Ok(_) => {
                                authenticated = true;
                                println!("Authentication successful");
                            }
                            Err(errstr) => {
                                println!("Failed to authenticate: {}", errstr);
                                break;
                            }
                        }
                    }

                    /* echo the data */
                    println!("Sending data back");
                    stream.write(&data[4..size]).unwrap();
                },

                /* If we received 0 bytes, we're done */
                Ok(_) => {
                    println!("Gracefully closing the connection with {}", stream.peer_addr().unwrap());
                    break;
                },

                Err(_) => {
                    println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                    stream.shutdown(Shutdown::Both).unwrap();
                    break;
                }
            }
        }
    }

}
