use std::thread;
use std::string::String;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::result::Result;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, Shutdown};
use std::time::Duration;
use std::collections::VecDeque;
use std::str::FromStr;

use crate::common::VERBOSITY;
use crate::common::verbose_print::VerbosityLevel;
use crate::{Error, Warn, Inform, Detail, Spam};
use crate::common::utils::encode_hex;
use crate::config::ProxifyConfig;
use crate::proxy_conn::ProxyConn;
use crate::proxy_conn::ProxyConnProtocol;
use crate::proxify_data::{ProxifyCommand, ProxifyDataType, ProxifyData};

/* To clarify the following type alias:
   A ref-counted thread-safe double-edge list containing ref-counted
   thread-safe elements */
type ThreadSafeList = Arc<Mutex<VecDeque<Arc<Mutex<ProxyConn>>>>>;

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

pub struct ProxifyDaemon {
    bind_addr: String,
    bind_port: u16,
    nr_of_proxies: u8,
    nr_of_prepare_threads: u8,
    notready_proxies: ThreadSafeList,
    ready_proxies: ThreadSafeList,
    inuse_proxies: ThreadSafeList,
}

/* Destructor */
impl Drop for ProxifyDaemon {
    fn drop(&mut self) {
        Inform!("Stopping listener");
    }
}

impl ProxifyDaemon {
    pub fn new(config: ProxifyConfig) -> Result<Self, String> {
        let mut proxies_list: VecDeque<Arc<Mutex<ProxyConn>>> = VecDeque::new();

        let mut id = 0_u16;
        for p in config.proxies_list {
            let prot: ProxyConnProtocol = match p.0.parse() {
                Ok(p) => p,
                Err(e) => return Err(format!("Failed to parse protocol {}: {}", p.0, e.to_string())),
            };
            proxies_list.push_back(Arc::new(Mutex::new(
                ProxyConn::new(id, p.0.parse().unwrap(), p.1, p.2)
            )));
            id += 1;
        }

        Spam!("Successfully parsed {} proxies from the configuration", id);

        Ok(ProxifyDaemon {
            bind_addr: config.bind_addr,
            bind_port: config.bind_port,
            nr_of_proxies: config.nr_of_proxies,
            nr_of_prepare_threads: config.nr_of_prepare_threads,
            notready_proxies: Arc::new(Mutex::new(proxies_list)),
            ready_proxies: Arc::new(Mutex::new(VecDeque::new())),
            inuse_proxies: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub fn get_bind_addr(&self) -> &String {
        &self.bind_addr
    }

    pub fn get_bind_port(&self) -> u16{
        self.bind_port
    }

    pub fn get_ready_proxy(&self) -> Option<Arc<Mutex<ProxyConn>>> {
       let mut r_proxies = self.ready_proxies.lock().unwrap();
        if r_proxies.is_empty() {
            Inform!("No proxies are ready yet.");
            return None;
        }
        let proxy = r_proxies.pop_front().unwrap();
        drop(r_proxies); // Am I dropping the Mutex or the Arc<Mutex<VecDeque<...>>>?
        let mut u_proxies = self.inuse_proxies.lock().unwrap();
        u_proxies.push_back(proxy.clone());
        Some(proxy)
    }

    /* Run in a separate thread. This thread will run forever with no
       interaction. It will exit if the argument "exiting" becomes True. */
    pub fn prepare_proxies(thread_nr: u8,
                           notready_proxies: ThreadSafeList,
                           ready_proxies: ThreadSafeList,
                           inuse_proxies: ThreadSafeList,
                           exiting: Arc<AtomicBool>) {
        Detail!("Thread {} is starting to prepare proxies", thread_nr);
        while !exiting.load(Ordering::Relaxed) {
            /* Process flow:
               Loop inuse and try_lock, if success then push_back to unused
               then if nr_proxies not reached pop_first from notready, make
               ready then push_back to ready_proxies */
            let mut notready_guard = notready_proxies.lock().unwrap();
            if notready_guard.is_empty() {
                Spam!("[prepare thread {}] No proxies to prepare, checking again in 1 second", thread_nr);
                drop(notready_guard);
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            let proxy_guard = notready_guard.pop_front().unwrap();
            drop(notready_guard);

            let mut proxy = proxy_guard.lock().unwrap();
            if let Err(e) = proxy.prepare() {
                Error!("[prepare thread {}] Failed to prepare proxy {}: {}", thread_nr, proxy.get_id(), e.to_string());
            }

            /* If it is prepared, add it to ready_proxies
               else push_back to notready_proxies */
            if proxy.is_prepared() {
                Spam!("[prepare thread {}] Proxy {} is now prepared", thread_nr, proxy.get_id());
                /* Intentionally not handling the error since it should never
                   happen */
                let mut ready_guard = ready_proxies.lock().unwrap();
                drop(proxy);
                ready_guard.push_back(proxy_guard);
            } else {
                Spam!("[prepare thread {}] Proxy {} failed to prepare", thread_nr, proxy.get_id());
                drop(proxy);
                let mut notready_guard = notready_proxies.lock().unwrap();
                notready_guard.push_back(proxy_guard);
            }
        }
        Spam!("Thread {} is exiting", thread_nr);
    }

    pub fn start(&mut self, exiting: &Arc<AtomicBool>) -> std::io::Result<()>{
        let listener = TcpListener::bind((self.bind_addr.as_str(), self.bind_port)).unwrap();
        let nr_threads: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let mut prepare_threads: Vec<thread::JoinHandle<_>> = Vec::new();

        /* Kick off a given number threads that will keep proxies prepared */
        Detail!("Preparing {} number of proxies using {} threads", self.nr_of_proxies, self.nr_of_prepare_threads);
        for thread_nr in 1..=self.nr_of_prepare_threads {
            let exiting_clone = exiting.clone();
            let notready_proxies_clone = self.notready_proxies.clone();
            let ready_proxies_clone = self.ready_proxies.clone();
            let inuse_proxies_clone = self.inuse_proxies.clone();
            Spam!("Starting prepare thread {}", thread_nr);
            prepare_threads.push(thread::spawn(move || {
                Self::prepare_proxies(thread_nr,
                                      notready_proxies_clone,
                                      ready_proxies_clone,
                                      inuse_proxies_clone,
                                      exiting_clone);
                })
            )
        }

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

        /* Wait for all prepare threads to finish */
        let mut threads_left = self.nr_of_prepare_threads;
        for pt in prepare_threads {
            Spam!("Waiting for {} prepare thread(s) to join", threads_left);
            pt.join().unwrap();
            threads_left -= 1;
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
                                Inform!("Authentication successful");
                            }
                            Err(errstr) => {
                                Error!("Failed to authenticate: {}", errstr);
                                break;
                            }
                        }
                    }

                    /* echo the data */
                    Detail!("Sending data back");
                    stream.write(&data[4..size]).unwrap();

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
                    let parsed_data = match ProxifyData::unmarshal_bytes(fake_data) {
                        Ok(d) => d,
                        Err(e) => {
                            Error!("Received invalid data from client: {}", e.to_string());
                            break;
                        },
                    };
                    Spam!("dummy request command u8 is {}", parsed_data.command as u8);
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
