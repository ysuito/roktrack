//! Provide Camera Function
//!

use rscam::{Camera, Config};
use std::fs;
use std::io::Write;

use crate::module::util::init::RoktrackProperty;

/// V4l2 Camera Setting
///
pub struct V4l2 {
    cap: Camera,
    property: RoktrackProperty,
}
/// V4l2 Camera methods
///
impl V4l2 {
    /// V4l2's constructor
    ///
    pub fn new(property: RoktrackProperty) -> Self {
        let mut cap = Camera::new("/dev/video0").unwrap();
        cap.start(&Config {
            interval: (1, 30), // 30 fps.
            resolution: (
                property.conf.camera.width as u32,
                property.conf.camera.height as u32,
            ),
            format: b"MJPG",
            nbuffers: 1,
            ..Default::default()
        })
        .unwrap();
        Self { cap, property }
    }
    /// Take a picture. Save two image.
    ///
    pub fn take(&self) {
        let _ = self.cap.capture(); // grab for reduce delay
        let frame = self.cap.capture().unwrap();
        // save original image
        let mut file = fs::File::create(self.property.path.img.last.clone()).unwrap();
        file.write_all(&frame[..]).unwrap();
    }
}
