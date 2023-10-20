//! Audio Handler.

// use soloud::*;
use async_process::Stdio;
use std::path::Path;
use std::thread::{self};

/// Play an audio file.
///
/// This function plays an audio file located at the specified path.
///
/// # Arguments
///
/// * `file` - A string representing the file path of the audio file.
///
/// # Example
///
/// ```
/// use roktracklib::module::device::speaker::play;
/// play("asset/audio/ja/start_mowing.mp3");
/// ```
pub fn play(file: &str, sync: bool) {
    let path = Path::new(file);
    if sync {
        std::process::Command::new("mpg123")
            .arg("-q")
            .arg(path.as_os_str())
            .output()
            .expect("failed to execute mpg123");
    } else {
        let _ = async_process::Command::new("mpg123")
            .arg("-q")
            .arg(path.as_os_str())
            .stdout(Stdio::piped())
            .spawn();
    }
}

/// Play an asset audio file.
///
/// This function plays an audio file located in the asset directory.
///
/// # Arguments
///
/// * `name` - A string representing the name of the asset audio file (without extension).
///
/// # Example
///
/// ```
/// speak("start_mowing");
/// ```
pub fn speak(name: &str) {
    let path = Path::new("./asset/audio/ja/").join(format!("{name}.mp3"));
    thread::spawn(move || play(path.to_str().unwrap(), false));
}

pub fn speak_sync(name: &str) {
    let path = Path::new("./asset/audio/ja/").join(format!("{name}.mp3"));
    thread::spawn(move || play(path.to_str().unwrap(), true));
}

/// Logger functions for speaking audio messages based on log levels.
pub mod logger {
    use super::speak;

    /// Log a debug message and speak the associated audio.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the audio file (without extension).
    /// * `level` - A string representing the log level (e.g., "DEBUG").
    ///
    /// # Example
    ///
    /// ```
    /// assert!(logger::debug("start_mowing", "DEBUG"));
    /// assert_eq!(logger::debug("start_mowing", "INFO"), false);
    /// ```
    pub fn debug(name: &str, level: &str) -> bool {
        if ["DEBUG"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// Log an info message and speak the associated audio.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the audio file (without extension).
    /// * `level` - A string representing the log level (e.g., "INFO").
    ///
    /// # Example
    ///
    /// ```
    /// assert!(logger::info("start_mowing", "INFO"));
    /// assert_eq!(logger::info("start_mowing", "WARNING"), false);
    /// ```
    pub fn info(name: &str, level: &str) -> bool {
        if ["DEBUG", "INFO"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// Log a warning message and speak the associated audio.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the audio file (without extension).
    /// * `level` - A string representing the log level (e.g., "WARN").
    ///
    /// # Example
    ///
    /// ```
    /// assert!(logger::warn("start_mowing", "WARN"));
    /// assert_eq!(logger::warn("start_mowing", "ERROR"), false);
    /// ```
    pub fn warn(name: &str, level: &str) -> bool {
        if ["DEBUG", "INFO", "WARN"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// Log an error message and speak the associated audio.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the audio file (without extension).
    /// * `level` - A string representing the log level (e.g., "ERROR").
    ///
    /// # Example
    ///
    /// ```
    /// assert!(logger::error("start_mowing", "ERROR"));
    /// assert!(logger::error("start_mowing", "DEBUG"));
    /// ```
    pub fn error(name: &str, level: &str) -> bool {
        if ["DEBUG", "INFO", "WARN", "ERROR"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_test() {
        play("asset/audio/ja/start_mowing.mp3", true);
        play("start_mowing", true);
        assert!(logger::debug("start_mowing", "DEBUG"));
        assert!(!logger::debug("start_mowing", "INFO"));
        assert!(logger::info("start_mowing", "INFO"));
        assert!(!logger::info("start_mowing", "WARNING"));
        assert!(logger::warn("start_mowing", "WARN"));
        assert!(!logger::warn("start_mowing", "ERROR"));
        assert!(logger::error("start_mowing", "ERROR"));
        assert!(logger::error("start_mowing", "DEBUG"));
    }
}
