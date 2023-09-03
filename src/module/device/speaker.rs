//! Audio Handler.
//!

use std::path::Path;

/// # play file
///
/// play file
///
/// ```
/// use roktracklib::module::device::speaker::play;
/// play("asset/audio/ja/start_mowing.mp3");
/// ```
///
pub fn play(file: &str) {
    let path = Path::new(file);
    let exist: bool = path.is_file();

    if exist {
        use soloud::*;

        let sl = Soloud::default().unwrap();

        let mut wav = audio::Wav::default();

        wav.load(std::path::Path::new(&file)).unwrap();

        sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
        while sl.voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// # speak
///
/// play asset files.
///
/// ```
/// use roktracklib::module::device::speaker::speak;
/// speak("start_mowing");
/// ```
///
pub fn speak(name: &str) {
    let path = crate::module::util::path::join(&["./asset/audio/ja/", &format!("{name}.mp3")]);
    play(path.as_str())
}

pub mod logger {
    use super::speak;
    /// ```
    /// use roktracklib::module::device::speaker::logger::debug;
    /// assert!(debug(&"start_mowing", &"DEBUG"));
    /// assert_eq!(debug(&"start_mowing", &"INFO"),false);
    /// ```
    ///
    pub fn debug(name: &str, level: &str) -> bool {
        if ["DEBUG"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// ```
    /// use roktracklib::module::device::speaker::logger::info;
    /// assert!(info(&"start_mowing", &"INFO"));
    /// assert_eq!(info(&"start_mowing", &"WARNING"),false);
    /// ```
    ///
    pub fn info(name: &str, level: &str) -> bool {
        if ["DEBUG", "INFO"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// ```
    /// use roktracklib::module::device::speaker::logger::warn;
    /// assert!(warn(&"start_mowing", &"WARN"));
    /// assert_eq!(warn(&"start_mowing", &"ERROR"),false);
    /// ```
    ///
    pub fn warn(name: &str, level: &str) -> bool {
        if ["DEBUG", "INFO", "WARN"].contains(&level) {
            speak(name);
            true
        } else {
            false
        }
    }

    /// ```
    /// use roktracklib::module::device::speaker::logger::error;
    /// assert!(error(&"start_mowing", &"ERROR"));
    /// assert!(error(&"start_mowing", &"DEBUG"));
    /// ```
    ///
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
