//! This module is responsible for preparing the resources needed by the application, such as directories, configurations, logs, etc.
//!

pub mod resource {
    use super::RoktrackProperty; // Import the RoktrackProperty type from the parent module

    /// Initialize the application resources and return a RoktrackProperty instance containing paths and configurations.
    ///
    pub fn init() -> RoktrackProperty {
        // Announce the start of mowing by calling the speak function from the speaker submodule
        crate::module::device::speaker::speak("start_mowing");

        // Prepare the app data directory by calling the create_app_sub_dir function from the dir submodule
        let paths = crate::module::util::path::dir::create_app_sub_dir();

        // Load the app configuration file by calling the load function from the toml submodule
        let conf =
            crate::module::util::conf::toml::load(&paths.dir.data).expect("Can't load config.");

        // Return a RoktrackProperty instance that contains the paths and configurations
        RoktrackProperty { path: paths, conf }
    }
}

/// This struct represents the properties of the app, such as paths and configurations.
///
#[derive(Debug, Clone)] // Derive some traits for this struct
pub struct RoktrackProperty {
    pub path: crate::module::util::path::RoktrackPath, // The paths of the app resources
    pub conf: crate::module::util::conf::Config,       // The configurations of the app
}
