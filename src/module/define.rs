//! Module for Constants and Paths Definitions
//!
//! This module defines various constants and paths used throughout the application.

/// System Constants
pub mod system {
    /// Name of the system
    pub const NAME: &str = "roktrack";
}

/// File Paths
pub mod path {

    // Persistent Data Directory
    pub const PERSISTENT_DIR: &str = "/data/";

    // Ephemeral Data Directory
    pub const EPHEMERAL_DIR: &str = "/run/user/1000/";

    // Image Directory
    pub const IMG_DIR: &str = "img";

    // Log Directory
    pub const LOG_DIR: &str = "log";

    // Configuration File
    pub const CONF_FILE: &str = "conf.toml";

    // Last Captured Image
    pub const LAST_IMAGE: &str = "vision.jpg";

    // Cropped Image
    pub const CROP_IMAGE: &str = "crop.jpg";

    // YOLOv8 Model (320x320)
    pub const PYLON_320_MODEL: &str = "asset/model/roktrack_yolov8_nano_fixed_320_320.onnx";

    // YOLOv8 Model (640x640)
    pub const PYLON_640_MODEL: &str = "asset/model/roktrack_yolov8_nano_fixed_640_640.onnx";

    // Digit Detection Model (96x96)
    pub const DIGIT_OCR_96_MODEL: &str = "asset/model/digit_yolov8_nano_fixed_96_96.onnx";

    // Animal Detection Model (320x320)
    pub const ANIMAL_320_MODEL: &str = "";

    // Animal Detection Model (640x640)
    pub const ANIMAL_640_MODEL: &str = "";
}
