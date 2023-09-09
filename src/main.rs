//! This module defines the main functionality of Roktrack, a marker-guided robotic mower.
//!
//! # Example
//! ![HowToWorkOneWay](https://raw.githubusercontent.com/ysuito/roktrack/master/asset/img/one_node_mowing.gif)
//!
//! Simply surround the area you want to mow with pylons, turn on the switch, and the grass is automatically mowed.
//!
//! # Warning
//! Fast-spinning lawnmower blades are very dangerous and can also eject debris at high speed.

// Import the module submodule that contains other modules
use crate::module::define; // Import the define module that contains constants
use crate::module::util::init::resource::init; // Import the resource initialization function
use log::LevelFilter; // Import the LevelFilter enum from the log crate
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender; // Import the FileAppender struct from the log4rs crate
use log4rs::config::{Appender, Config, Root}; // Import the Appender, Config, and Root structs from the log4rs crate
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use std::env;
use std::path::Path; // Import the PatternEncoder struct from the log4rs crate

pub mod module;

/// The main function of Roktrack
pub fn main() {
    // handle command line args
    let args: Vec<String> = env::args().collect();
    let mut console_level = LevelFilter::Warn;
    if args.len() > 1 && args[1] == "debug" {
        console_level = LevelFilter::Debug;
    }

    // Prepare the resources by initializing the property struct
    let property = init();

    // Initialize the logging system with the data directory and the system name
    init_log(
        property.path.dir.data.as_str(),
        define::system::NAME,
        console_level,
    );
    log::info!("Starting Roktrack..."); // Log an info message

    // Start the drive thread that controls the movement of the mower
    let drive_handler = module::drive::run(property);

    // Wait for the drive thread to finish before exiting the main function
    let _ = drive_handler.join();
}

/// This function initializes the logger system using the log4rs crate.
///
/// # Arguments
/// * `dir` - A string slice that holds the directory where the log file will be stored
/// * `name` - A string slice that holds the name of the logger and the log file
///
/// # Example
/// ```
/// init_log("./log_dir", "logger_name"); // Initialize the logger with the given directory and name
/// ```
///
/// # Log Example
/// ```
/// log::debug!("Debug Message"); // Log a debug message
/// log::info!("Info Message"); // Log an info message
/// log::warn!("Warning Message"); // Log a warning message
/// log::error!("Error Message"); // Log an error message
/// ```
fn init_log(dir: &str, name: &str, console_level: LevelFilter) {
    // File Handler
    let logfile = FileAppender::builder() // Create a new FileAppender builder
        .encoder(Box::new(PatternEncoder::new(
            // Set the encoder to a new PatternEncoder with a custom format
            "{h({d} - {l}: {m}{n})}",
        )))
        .build(
            Path::new(dir)
                .join(define::path::LOG_DIR)
                .join(format!("{}.log", name)),
        )
        .expect("Log file initialization error"); // Unwrap the result or panic if there is an error

    // Stdout Handler
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({d} - {l}: {m}{n})}")))
        .build();

    // Log config
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_level)))
                .build("console", Box::new(stdout)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build("logfile", Box::new(logfile)),
        )
        .build(
            Root::builder()
                .appender("console")
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, warn};
    use std::fs;
    use std::path::Path;

    // A simple test case for the init_log function
    #[test]
    fn test_log() {
        // Define a test directory and name
        let dir = "/tmp/roktracktest/";
        let name = "test_log";

        // Call the init_log function
        init_log(dir, name, LevelFilter::Debug);

        // Perform some logging
        debug!("Debug Message");
        info!("Info Message");
        warn!("Warning Message");
        error!("Error Message");

        // Read the contents of the log file
        let log_file_path_str = "/tmp/roktracktest/log/test_log.log";
        let log_file_path = Path::new(log_file_path_str);
        let log_contents = fs::read_to_string(log_file_path).expect("Failed to read log file");

        // Assert that log messages are present in the file
        assert!(!log_contents.contains("Debug Message"));
        assert!(log_contents.contains("Info Message"));
        assert!(log_contents.contains("Warning Message"));
        assert!(log_contents.contains("Error Message"));
    }
}
