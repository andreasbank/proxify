use std::error::Error;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use clap::{arg, command};
use once_cell::sync::Lazy;
use ctrlc;

use proxify::common::verbose_print::{VerbosityLevel, Verbosity};
use proxify::{Error, Inform, Detail, Spam};
mod daemon;
use daemon::ProxifyDaemon;
mod config;
use config::ProxifyConfig;

static VERBOSITY: Lazy<Mutex<Verbosity>> = Lazy::new(|| Mutex::new(Verbosity::new()));
static EXITING: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_args = command!().args(&[
        arg!(-d --debug <lvl> "Enable debug at a certain level"),
        arg!(-c --config <string> "Start listening for proxify data with the given configuration"),
    ]).get_matches();

    let dbg_lvl = match cmd_args.get_one::<String>("debug") {
        Some(v) => match String::from(v).trim().parse::<u32>() {
            Ok(n) => n,
            Err(_e) => 0, // Move this error checking to the command!().
        },
        None => 0,
    };

    match dbg_lvl {
        0 => VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Quiet),
        1 => VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Errors),
        2 => VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Informative),
        3 => VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Detailed),
        4 => VERBOSITY.lock().unwrap().set_level(VerbosityLevel::Spam),
        5..=u32::MAX => {
            eprintln!("Invalid debug level {}, defaulting to 1 (Errors)", dbg_lvl);
        },
    }
    Inform!("Debug level is {} ({})", dbg_lvl, VERBOSITY.lock().unwrap());

    let arg_conf = match cmd_args.get_one::<String>("config") {
        Some(conf) => String::from(conf),
        None => String::from(""),
    };
    Detail!("Command line configuration string: '{}'", arg_conf);

    let conf = ProxifyConfig::new(&arg_conf)?;
    Inform!("Configuration: '{}'", "<TODO>");

    let mut listener: ProxifyDaemon = match ProxifyDaemon::new(conf) {
        Ok(lis) => lis,
        Err(e) => {
            return Err(e.into());
        }
    };

    {
        /* Handle CTRL-C sigterm */
        let exiting_clone = EXITING.clone();
        let listener_addr = listener.get_bind_addr().clone();
        let listener_port = listener.get_bind_port();
        ctrlc::set_handler(move || {
            Inform!("Caught sigterm, exiting...");
            exiting_clone.store(true, Ordering::SeqCst);
            /* We connect to the listening server to wake it up and see that we are exiting */
            let _ = TcpStream::connect((listener_addr.as_str(), listener_port));
        })
        .expect("Error setting Ctrl+C handler");
    }

    Inform!("Listening on {}:{}", listener.get_bind_addr(), listener.get_bind_port());
    listener.start(once_cell::sync::Lazy::<Arc<AtomicBool>>::get(&EXITING).unwrap())?;

    Ok(())
}
