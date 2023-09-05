//! Camera Functions
//!

use rscam::{Camera, Config};
use std::fs;
use std::io::Write;

use crate::module::util::init::RoktrackProperty;

/// Represents a V4L2 camera configuration and capture functionality.
///
pub struct V4l2Camera {
    cap: Camera,                // The camera instance for capturing frames.
    property: RoktrackProperty, // Configuration properties for the camera.
}

impl V4l2Camera {
    /// Creates a new V4L2 camera instance with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `property` - The camera configuration properties.
    ///
    /// # Returns
    ///
    /// A `V4l2Camera` instance.
    ///
    pub fn new(property: RoktrackProperty) -> Self {
        let mut cap = Camera::new("/dev/video0").unwrap();

        // Configure and start the camera with specified settings.
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

    /// Captures a frame from the camera and saves it to a file.
    ///
    /// This method captures a frame from the camera and saves it to a file specified
    /// in the `RoktrackProperty`. The images are saved with a specific filename format.
    pub fn take_picture(&self) {
        let _ = self.cap.capture(); // Grab a frame to reduce delay.
        let frame = self.cap.capture().unwrap();

        // Save the original image to the specified file path.
        let mut file = fs::File::create(self.property.path.img.last.clone()).unwrap();
        file.write_all(&frame[..]).unwrap();
    }
}
