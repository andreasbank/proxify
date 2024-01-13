/*
USAGE:

use std::sync::{Mutex, Arc};

// Create a global state wrapped in a Mutex
static VERBOSITY: Mutex<Verbosity> = Mutex::new(Verbosity::new)));

// Set the global state flag to  Errors (or any other VerbosityLevel)
VERBOSITY.lock().unwrap().set_value(VerbosityLevel::Errors);

// Use these macro to conditionally print text depending on the VERBOSITY variable
Error("This text will be printed  only for VerbosityLevel::Errors");
Informative("This text will be printed  only for VerbosityLevel::Informative");
Detailed("This text will be printed  only for VerbosityLevel::Detailed");
Spam("This text will be printed  only for VerbosityLevel::Spam");
*/

use std::fmt;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum VerbosityLevel {
    Quiet,
    Errors,
    Informative,
    Detailed,
    Spam
}

// Define a struct to encapsulate the global state
pub struct Verbosity{
    level: VerbosityLevel,
}

// Implement methods for the struct to safely modify the state
impl Verbosity {
    pub fn new() -> Self {
        Verbosity { level: VerbosityLevel::Quiet }
    }

    pub fn set_level(&mut self, value: VerbosityLevel) {
        self.level = value;
    }

    pub fn is_atleast_level(&self, level: VerbosityLevel) -> bool {
        if level <= self.level {
            return true;
        }
        false
    }
}

impl fmt::Display for Verbosity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.level)
    }
}

// Define the macros to accept arguments like println!
#[macro_export]
macro_rules! Error {
    ($($arg:tt)*) => {
        {
            // Check the global state
            if VERBOSITY.lock().unwrap().is_atleast_level(VerbosityLevel::Errors) {
                println!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! Inform {
    ($($arg:tt)*) => {
        {
            // Check the global state
            if VERBOSITY.lock().unwrap().is_atleast_level(VerbosityLevel::Informative) {
                println!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! Detail {
    ($($arg:tt)*) => {
        {
            // Check the global state
            if VERBOSITY.lock().unwrap().is_atleast_level(VerbosityLevel::Detailed) {
                println!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! Spam {
    ($($arg:tt)*) => {
        {
            // Check the global state
            if VERBOSITY.lock().unwrap().is_atleast_level(VerbosityLevel::Spam) {
                println!($($arg)*);
            }
        }
    };
}
