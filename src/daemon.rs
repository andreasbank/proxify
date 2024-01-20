use std::thread;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::result::Result;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, Shutdown};
use std::time::Duration;

use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::utils::encode_hex;
use crate::VERBOSITY;
use crate::config::ProxifyConfig;
use proxify::proxy_conn::ProxyConn;

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

pub struct ProxifyDaemon {
    bind_addr: String,
    bind_port: u16,
    nr_of_proxies: u16,
}

/* Destructor */
impl Drop for ProxifyDaemon {
    fn drop(&mut self) {
        Inform!("Stopping listener");
    }
}

impl ProxifyDaemon {
    pub fn new(config: ProxifyConfig) -> Result<Self, String> {
        Ok(ProxifyDaemon {
            bind_addr: config.bind_addr,
            bind_port: config.bind_port,
            nr_of_proxies: 20,
        })
    }

    pub fn get_bind_addr(&self) -> &String {
        &self.bind_addr
    }

    pub fn get_bind_port(&self) -> u16{
        self.bind_port
    }

    pub fn prepare_proxies(&mut self) {
        // TODO: Write code that prepares a number of proxies
        Detail!("Preparing {} number of proxies", self.nr_of_proxies);
    }

    pub fn start(&mut self, exiting: &Arc<AtomicBool>) -> std::io::Result<()>{
        self.prepare_proxies();
        let listener = TcpListener::bind((self.bind_addr.as_str(), self.bind_port)).unwrap();
        let nr_threads: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

        /* More on a proper implementation of TcpListener::incoming():
           https://stackoverflow.com/questions/56692961/graceful-exit-tcplistener-incoming */
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
                Err(e) => {
                    Error!("Failed to accept incoming connection: {}", e);
                }
            }
        }
        Ok(())
    }

    /* A very simple check to ensure the client is compatible */
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

                    // TODO: parse proxify data struct
                    // TODO: if command is do_request with new session, get new proxy
                    // TODO: do request with given data
                    // TODO: write back data to stream

                },

                /* If we received 0 bytes, we're done */
                Ok(_) => {
                    Detail!("Gracefully closing the connection with {}", stream.peer_addr().unwrap());
                    break;
                },

                Err(_) => {
                    Error!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                    break;
                }
            }
        }
        *nr_threads.lock().unwrap() -= 1;
    }
}
