//! This module defines the main functionality of Roktrack, a marker-guided robotic mower.

pub mod module; // Import the module submodule that contains other modules
use crate::module::define; // Import the define module that contains constants and types
use crate::module::util::init::resource::init; // Import the resource initialization function

// The main function of Roktrack
pub fn main() {
    // Prepare the resources by initializing the property struct
    let property = init();

    // Initialize the logging system with the data directory and the system name
    init_log(property.path.dir.data.as_str(), define::system::NAME);
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
fn init_log(dir: &str, name: &str) {
    use crate::module::util::path::join; // Import the join function from the path module
    use log::LevelFilter; // Import the LevelFilter enum from the log crate
    use log4rs::append::file::FileAppender; // Import the FileAppender struct from the log4rs crate
    use log4rs::config::{Appender, Config, Root}; // Import the Appender, Config, and Root structs from the log4rs crate
    use log4rs::encode::pattern::PatternEncoder; // Import the PatternEncoder struct from the log4rs crate

    let logfile = FileAppender::builder() // Create a new FileAppender builder
        .encoder(Box::new(PatternEncoder::new(
            // Set the encoder to a new PatternEncoder with a custom format
            "{h({d} - {l}: {m}{n})}",
        )))
        .build(join(&[
            // Build the FileAppender with the joined path of the directory, the log directory, and the name
            dir,
            define::path::LOG_DIR,
            &format!("{}.log", name),
        ]))
        .unwrap(); // Unwrap the result or panic if there is an error

    let config = Config::builder() // Create a new Config builder
        .appender(Appender::builder().build("logfile", Box::new(logfile))) // Add an appender with the name "logfile" and the FileAppender as a boxed trait object
        .build(Root::builder().appender("logfile").build(LevelFilter::Info)) // Build the Config with a Root that uses the "logfile" appender and has a level filter of Info
        .unwrap(); // Unwrap the result or panic if there is an error
    log4rs::init_config(config).unwrap(); // Initialize the logger system with the Config or panic if there is an error
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
        init_log(dir, name);

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
