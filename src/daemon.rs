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

use proxify::common::verbose_print::VerbosityLevel;
use proxify::{Error, Inform, Detail, Spam};
use proxify::common::utils::encode_hex;
use crate::VERBOSITY;
use crate::config::ProxifyConfig;
use crate::proxy_conn::ProxyConn;
use crate::proxy_conn::ProxyConnProtocol;

/* To clarify the following type alias:
   A ref-counted thread-safe double-edge list containing ref-counted
   thread-safe elements */
type ThreadSafeList = Arc<Mutex<VecDeque<Arc<Mutex<ProxyConn>>>>>;

static MAGIC_BYTES: [u8; 4] = [ 0xAB, 0xBA, 0xAB, 0xBA ];

pub struct ProxifyDaemon {
    bind_addr: String,
    bind_port: u16,
    nr_of_proxies: u8,
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
    pub fn prepare_proxies(notready_proxies: ThreadSafeList,
                           ready_proxies: ThreadSafeList,
                           inuse_proxies: ThreadSafeList,
                           exiting: Arc<AtomicBool>) {
        Detail!("Starting to prepare proxies");
        while !exiting.load(Ordering::Relaxed) {
            Spam!("In prepare proxy loop");

            /*  TODO:
                Process flow:
                Loop inuse and try_lock, if success then push_back to unused
                then if nr_proxies not reached pop_first from notready, make ready
                then push_back to ready_proxies
             */
            let mut proxy_list = notready_proxies.lock().unwrap();
            if proxy_list.is_empty() {
                Spam!("No proxies to prepare, checking again in 1 second");
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            let mut proxy_guard = proxy_list.pop_front().unwrap();
            let mut proxy = proxy_guard.lock().unwrap();
            if let Err(e) = proxy.prepare() {
                Error!("Failed to prepare proxy {}", proxy.get_id());
            }

            /* If it is prepared, add it to ready_proxies
               else push_back to notready_proxies */
            if proxy.is_prepared() {
                Spam!("Proxu {} is now prepared", proxy.get_id());
                /* Intentionally not handling the error since it should never
                   happen */
                // TODO: continue here!
                //ready_proxies.lock().unwrap().push_back(proxy);
            }
        }
    }

    pub fn start(&mut self, exiting: &Arc<AtomicBool>) -> std::io::Result<()>{
        let listener = TcpListener::bind((self.bind_addr.as_str(), self.bind_port)).unwrap();
        let nr_threads: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

        /* Kick off a thread that will keep proxies prepared */
        Detail!("Preparing {} number of proxies", self.nr_of_proxies);
        let exiting_clone = exiting.clone();
        let notready_proxies_clone = self.notready_proxies.clone();
        let ready_proxies_clone = self.ready_proxies.clone();
        let inuse_proxies_clone = self.inuse_proxies.clone();
        let proxies_thread = thread::spawn(move || {
            Self::prepare_proxies(notready_proxies_clone,
                                  ready_proxies_clone,
                                  inuse_proxies_clone,
                                  exiting_clone);
        });

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
        proxies_thread.join().unwrap();
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
