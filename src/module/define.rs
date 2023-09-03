//! DEFINE
//!

pub mod system {
    pub const NAME: &str = "roktrack";
}
pub mod path {
    pub const PERSISTENT_DIR: &str = "/data/";
    pub const EPHEMERAL_DIR: &str = "/run/user/1000/";
    pub const IMG_DIR: &str = "img";
    pub const LOG_DIR: &str = "log";
    pub const CONF_FILE: &str = "conf.toml";
    pub const LAST_IMAGE: &str = "vision.jpg";
    pub const CROP_IMAGE: &str = "crop.jpg";
    pub const PYLON_320_MODEL: &str = "asset/model/roktrack_yolov8_nano_fixed_320_320.onnx";
    pub const PYLON_640_MODEL: &str = "asset/model/roktrack_yolov8_nano_fixed_640_640.onnx";
    pub const DIGIT_OCR_96_MODEL: &str = "asset/model/digit_yolov8_nano_fixed_96_96.onnx";
    pub const ANIMAL_320_MODEL: &str = "";
    pub const ANIMAL_640_MODEL: &str = "";
}
