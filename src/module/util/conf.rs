//! Config Handler.

use serde::{Deserialize, Serialize};

/// Provides TOML config file handling.
pub mod toml {

    use super::DEFAULT_CONFIG;
    use crate::module::define;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    /// Loads a configuration file from the given directory.
    /// If not found, generates a default config file.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory where the configuration file is located or should be created.
    ///
    pub fn load(dir: &str) -> super::Config {
        // Check if the config file exists
        let path = Path::new(dir).join(define::path::CONF_FILE);
        let exist: bool = path.is_file();

        if !exist {
            // Create the default config if it doesn't exist
            let config: super::Config = toml::from_str(DEFAULT_CONFIG).unwrap();
            let toml_str = toml::to_string(&config).unwrap();
            let mut file = File::create(&path).unwrap();
            file.write_all(toml_str.as_bytes()).unwrap();
        }

        // Load the config
        let conf_str: String = std::fs::read_to_string(&path).unwrap();
        let setting: Result<super::Config, toml::de::Error> = toml::from_str(&conf_str);

        match setting {
            Ok(conf) => conf,
            Err(e) => panic!("Failed to parse TOML: {}", e),
        }
    }

    /// Saves a configuration file to the given directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory where the configuration file should be saved.
    /// * `conf` - The configuration data to be saved.
    ///
    pub fn save(dir: &str, conf: super::Config) {
        let toml_str = toml::to_string(&conf).unwrap();
        let path = crate::module::util::path::join(&[dir, define::path::CONF_FILE]);
        let mut file = File::create(path).unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
    }
}

/// Represents the configuration data structure.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub system: System,
    pub drive: Drive,
    pub camera: Camera,
    pub pin: Pin,
    pub pwm: Pwm,
    pub vision: Vision,
    pub notification: Notification,
    pub detectthreshold: DetectThreshold,
}

/// Represents system-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct System {
    pub persistent_dir: String,
    pub ephemeral_dir: String,
    pub log_speaker_level: String,
    pub lang: String,
}

/// Represents drive-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Drive {
    pub default_state: String,
    pub mode: String,
    pub minimum_pylon_height: u16,
    pub turn_adj: f32,
    pub motor_driver: String,
}

/// Represents camera-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Camera {
    pub video_idx: i8,
    pub grab_times: u8,
    pub width: u16,
    pub height: u16,
}

/// Represents pin-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pin {
    pub left_pin1: u8,
    pub left_pin2: u8,
    pub right_pin1: u8,
    pub right_pin2: u8,
    pub bumper_pin: u8,
    pub work1_pin: u8,
    pub work2_pin: u8,
    pub work_ctrl_positive: bool,
}

/// Represents PWM-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pwm {
    pub pwm_power_left: f64,
    pub pwm_power_right: f64,
}

/// Represents vision-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vision {
    pub detector: String,
    pub ocr: bool,
}

/// Represents notification-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Notification {
    pub line_notify_token: String,
}

/// Represents detection threshold-related configuration parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DetectThreshold {
    pub pylon: f32,
    pub person: f32,
    pub animal: f32,
    pub roktrack: f32,
}

// Default configuration data in TOML format
const DEFAULT_CONFIG: &str = r#"
[system]
  persistent_dir = '/data/roktrack' # Directory for persistent data (Development)
  ephemeral_dir = '/run/user/1000/roktrack' # Directory for ephemeral data
  log_speaker_level = 'INFO' # Log speaker level (e.g., 'INFO', 'DEBUG')
  lang = 'ja' # Language setting ('ja' for Japanese, 'en' for English)

[drive]
  default_state = 'on' # Default state of the drive ('on' or 'off')
  mode = 'fill' # Drive mode ('fill', 'oneway', 'climb')
  minimum_pylon_height = 0 # Minimum pylon height for operations
  turn_adj = 1 # Turn adjustment factor
  motor_driver = 'ZK_5AD' # Motor driver type ('ZK_5AD', 'IRF3205')

[camera]
  video_idx = -1 # Video index (-1 for default)
  grab_times = 3 # Number of image grabs
  width = 1280 # Image width
  height = 720 # Image height

[pin]
  left_pin1 = 22 # Left motor control pin 1 (DIGITAL)
  left_pin2 =  23 # Left motor control pin 2 (PWM)
  right_pin1 = 24 # Right motor control pin 1 (DIGITAL)
  right_pin2 = 25 # Right motor control pin 2 (PWM)
  bumper_pin = 26 # Bumper sensor pin
  work1_pin = 14 # Work motor control pin 1 (for relay, use 17)
  work2_pin = 18 # Work motor control pin 2
  work_ctrl_positive = false # Work motor control polarity (for relay, set to true)

[pwm]
  pwm_power_left = 1.0 # PWM power for the left motor (in percentage)
  pwm_power_right = 1.0 # PWM power for the right motor (in percentage)

[vision]
  detector = 'yolov7onnx' # Object detection model ('yolov7onnx', deprecated models)
  ocr = true # Enable optical character recognition (OCR)

[notification]
  line_notify_token = 'YOUR-LINE-NOTIFY-TOKEN' # Line Notify token for notifications

[detectthreshold]
  pylon = 0 # Detection threshold for pylons
  person = 0.7 # Detection threshold for people
  animal = 0 # Detection threshold for animals
  roktrack = 0.5 # Detection threshold for Roktrack objects
"#;

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::Path;

    #[test]
    fn run_load() {
        fs::create_dir_all(Path::new("/tmp/roktracktest/")).unwrap();
        let res = toml::load("/tmp/roktracktest/");
        assert_eq!(res.system.lang, "ja");
    }
}
