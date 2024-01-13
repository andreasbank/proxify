use std::thread;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::result::Result;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::time::Duration;

use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::utils::encode_hex;
use crate::VERBOSITY;

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

pub struct ProxifyDaemon {
    addr: String,
    port: u16,
}

/* Destructor */
impl Drop for ProxifyDaemon {
    fn drop(&mut self) {
        Inform!("Stopping listener");
    }
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
        })
    }

    pub fn start(&mut self, exiting: &Arc<AtomicBool>) -> std::io::Result<()>{
        let listener = TcpListener::bind((self.addr.as_str(), self.port)).unwrap();
        let nr_threads: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

        for stream in listener.incoming() {
            if exiting.load(Ordering::Relaxed) {
                /* If the application is exiting break the loop */
                break;
            }
            match stream {
                Ok(stream) => {
                    if *nr_threads.lock().unwrap() >= 50 {
                        Inform!("Too many threads running, ignoring connections for 1 second");
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                    Inform!("Accepted connection from address {}", stream.peer_addr().unwrap());
                    let exiting_clone = exiting.clone();
                    let nr_threads_clone = nr_threads.clone();
                    *nr_threads.lock().unwrap() += 1;
                    thread::spawn(move|| {
                        Self::handle_accept(stream, exiting_clone, nr_threads_clone)
                    });
                }
                //None => {
                //    /* No connection, lets sleep for 1 second */
                //    thread::sleep(Duration::from_secs(1));
                //}
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    /* Blocking, continue loop */
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Ok(())
    }

    fn authenticate(data: &[u8]) -> Result<(), &'static str> {
        Spam!("Magic bytes received: {}", encode_hex(data));
        if MAGIC_BYTES == data { return Ok(()); }

        Err("Wrong magic bytes")
    }

    fn handle_accept(mut stream: TcpStream, exiting: Arc<AtomicBool>, nr_threads: Arc<Mutex<i32>>) {
        let mut authenticated = false;
        let mut data = [0 as u8; 1024];

        Detail!("Thread {} is running", nr_threads.lock().unwrap());

        while !exiting.load(Ordering::Relaxed) {
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
                    Detail!("Sending data back");
                    stream.write(&data[4..size]).unwrap();
                },

                /* If we received 0 bytes, we're done */
                Ok(_) => {
                    Detail!("Gracefully closing the connection with {}", stream.peer_addr().unwrap());
                    break;
                },

                Err(_) => {
                    Error!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                    stream.shutdown(Shutdown::Both).unwrap();
                    break;
                }
            }
        }
        *nr_threads.lock().unwrap() -= 1;
    }

}
