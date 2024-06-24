pub mod verbose_print;
pub mod utils;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use verbose_print::Verbosity;
pub static VERBOSITY: Lazy<Mutex<Verbosity>> = Lazy::new(|| Mutex::new(Verbosity::new()));