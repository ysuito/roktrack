//! Prepare resources such as directories, configurations, logs, etc.
//!

pub mod resource {
    use super::RoktrackProperty;

    /// # init
    ///
    /// Prepair app resources.
    ///
    pub fn init() -> RoktrackProperty {
        crate::module::device::speaker::speak("start_mowing");
        // Prepair app data directory.
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        // Load app conf file.
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        // Return Property
        RoktrackProperty { path: paths, conf }
    }
}
/// Retain property for this app.
///
#[derive(Debug, Clone)]
pub struct RoktrackProperty {
    pub path: crate::module::util::path::RoktrackPath,
    pub conf: crate::module::util::conf::Config,
}
