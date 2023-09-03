//! Roktrack. A marker-guided robotic mower.  
//!
pub mod module;

use crate::module::define;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Main loop of roktrack.
///
/// This main function launch two threads.
///
/// - Com thread for receiving ble advertisement
/// - Drive thread for auto pilot
///
pub fn main() {
    // Prepair Resources.
    let property = module::util::init::resource::init();

    // Log Initialization
    init_log(property.path.dir.data.as_str(), define::system::NAME);
    log::info!("Starting Roktrack...");

    // Prep Neighbor Table
    let neighbor_table: Arc<Mutex<HashMap<u8, module::com::Neighbor>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Start Com Thread
    let com = module::com::BleBroadCast::new();
    let com_listner = com.listen(neighbor_table.clone());

    // Start Drive Thread
    let drive_handler = module::drive::run(Arc::clone(&neighbor_table), property, com);

    // Join Thread
    let _ = com_listner.join();
    let _ = drive_handler.join();
}

/// Initialize Logger
///
/// # Example
/// ```
/// init_log("./log_dir", "logger_name");
/// ```
///
/// # Log Example
/// ```
/// log::debug!("Debug Message");
/// log::info!("Info Message");
/// log::warn!("Warning Message");
/// log::error!("Error Message");
/// ```
fn init_log(dir: &str, name: &str) {
    use crate::module::util::path::join;
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}",
        )))
        .build(join(&[
            dir,
            define::path::LOG_DIR,
            &format!("{}.log", name),
        ]))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
}
