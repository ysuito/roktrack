//! Processing related to visual information.
//!
use super::util::init::RoktrackProperty;

pub mod camera;
pub mod detector;

/// Provide a means of image processing.
///
pub struct RoktrackVision {
    pub cam: camera::V4l2,
    pub det: detector::onnx::YoloV8,
}

/// RoktrackVision's methods.
///
impl RoktrackVision {
    pub fn new(property: RoktrackProperty) -> Self {
        Self {
            cam: camera::V4l2::new(property),
            det: detector::onnx::YoloV8::new(),
        }
    }
}
