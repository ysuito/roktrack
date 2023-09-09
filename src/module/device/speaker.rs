//! Audio Handler.

use soloud::*;
use std::path::Path;

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
pub fn play(file: &str) {
    let path = Path::new(file);

    // Check if the file exists
    if path.is_file() {
        let sl = Soloud::default().unwrap();
        let mut wav = audio::Wav::default();

        // Load the audio file
        if let Err(error) = wav.load(path) {
            eprintln!("Failed to load audio file: {:?}", error);
            return;
        }

        sl.play(&wav);

        // Wait for the audio to finish playing
        while sl.voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
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
    play(path.to_str().unwrap())
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
        play("asset/audio/ja/start_mowing.mp3");
        play("start_mowing");
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
