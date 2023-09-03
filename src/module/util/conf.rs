//! Config Handler.
//!

use serde::{Deserialize, Serialize};

pub mod toml {
    //! Provide toml config file handling.
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    use crate::module::{define, util::conf::DEFAULT_CONFIG};

    // https://smile-jsp.hateblo.jp/entry/2020/04/20/170000

    /// # Load config file
    ///
    /// Reads a configuration file.
    /// If not found, generate a default config file.
    ///
    pub fn load(dir: &str) -> super::Config {
        // check config file existence
        let path = Path::new(dir).join(define::path::CONF_FILE);
        let exist: bool = path.is_file();
        if !exist {
            // create default config
            let config: super::Config = toml::from_str(DEFAULT_CONFIG).unwrap();
            let toml_str = toml::to_string(&config).unwrap();
            let mut file = File::create(&path).unwrap();
            file.write_all(toml_str.as_bytes()).unwrap();
        }
        // load config
        let conf_str: String = std::fs::read_to_string(&path).unwrap();
        let setting: Result<super::Config, toml::de::Error> = toml::from_str(&conf_str);
        match setting {
            Ok(conf) => conf,
            Err(e) => panic!("Filed to parse TOML: {}", e),
        }
    }
    /// # Save config file
    ///
    /// Save a configuration file.
    ///
    pub fn save(dir: &str, conf: super::Config) {
        let toml_str = toml::to_string(&conf).unwrap();
        let path = crate::module::util::path::join(&[dir, define::path::CONF_FILE]);
        let mut file = File::create(path).unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
    }
}
/// Retain config.
///
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
/// Retain system parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct System {
    pub persistent_dir: String,
    pub ephemeral_dir: String,
    pub log_speaker_level: String,
    pub lang: String,
}
/// Retain drive parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Drive {
    pub default_state: String,
    pub mode: String,
    pub minimum_pylon_height: u16,
    pub turn_adj: f32,
    pub motor_driver: String,
}
/// Retain camera parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Camera {
    pub video_idx: i8,
    pub grab_times: u8,
    pub width: u16,
    pub height: u16,
}
/// Retain pin parameter
///
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
/// Retain pwm parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pwm {
    pub pwm_power_left: f64,
    pub pwm_power_right: f64,
}
/// Retain vision parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vision {
    pub detector: String,
    pub ocr: bool,
}
/// Retain notification parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Notification {
    pub line_notify_token: String,
}
/// Retain detection threshold parameter
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DetectThreshold {
    pub pylon: f32,
    pub person: f32,
    pub animal: f32,
    pub roktrack: f32,
}
// default config
const DEFAULT_CONFIG: &str = r#"
[system]
  persistent_dir = '/data/roktrack' # For Development
  ephemeral_dir = '/run/user/1000/roktrack' # '/run/user/1000/roktrack's
  log_speaker_level = 'INFO'
  lang = 'ja' # ja, en
[drive]
  default_state = 'on' # on, off
  mode = 'fill' # fill, oneway, climb
  minimum_pylon_height = 0
  turn_adj = 1
  motor_driver = 'ZK_5AD' # ZK_5AD(default), IRF3205
[camera]
  video_idx = -1
  grab_times = 3
  width = 1280
  height = 720
[pin]
  left_pin1 = 22 # DIGITAL
  left_pin2 =  23 # PWM
  right_pin1 = 24 # DIGITAL
  right_pin2 = 25 # PWM
  bumper_pin = 26
  work1_pin = 14 # 正理論のリレーの場合は、17にする
  work2_pin = 18
  work_ctrl_positive = false # 正理論のリレーの場合は、Trueとする
[pwm]
  pwm_power_left = 1.0 # %
  pwm_power_right = 1.0 # %
[vision]
  detector = 'yolov7onnx' # yolov7onnx, yolox(deplicated), yolov7(deplicated)
  ocr = true
[notification]
  line_notify_token = 'YOUR-LINE-NOTIFY-TOKEN'
[detectthreshold]
  pylon = 0
  person = 0.7
  animal = 0
  roktrack = 0.5
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
