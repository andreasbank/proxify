use std::sync::{Mutex};
use std::error::Error;
use clap::{arg, command};
use once_cell::sync::Lazy;

use proxify::common::verbose_print::{VerbosityLevel, Verbosity};
use proxify::{Error, Inform, Detail, Spam};
mod daemon;
use daemon::ProxifyDaemon;
mod config;
use config::ProxifyConfig;

static VERBOSITY: Lazy<Mutex<Verbosity>> = Lazy::new(|| Mutex::new(Verbosity::new()));

fn main() -> Result<(), Box<dyn Error>> {
    let mut listener = ProxifyDaemon::new(&"127.0.0.1".to_string(), 65432).unwrap();
    let cmd_args = command!().args(&[
        arg!(-d --debug <lvl> "Enable debug at a certain level"),
        arg!(-c --config <string> "Start listening for proxify data with the given configuration"),
    ]).get_matches();

    // Remove'_' from _dbg_lvl when this is used
    let dbg_lvl = match cmd_args.get_one::<String>("debug") {
        Some(v) => match String::from(v).trim().parse::<u32>() {
            Ok(n) => {
                n
            },
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

    let conf = ProxyfyConfig::new(arg_conf);
    Inform!("Configuration: '{}'", arg_conf);

    Inform!("Listening on port 65432");
    listener.start()?;

    // This will never be reached
    listener.stop();

    Ok(())
}
