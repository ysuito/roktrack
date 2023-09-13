//! Monitoring Animal Pilot

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::Roktrack,
    pilot::base,
    pilot::RoktrackState,
    util::{common::send_line_notify_with_image, init::RoktrackProperty},
    vision::detector::{AnimalClasses, Detection},
    vision::VisionMgmtCommand,
};

pub struct MonitorAnimal {
    last_detected_time: u64,
}

impl MonitorAnimal {
    pub fn new() -> Self {
        Self {
            last_detected_time: 0,
        }
    }
}

impl Default for MonitorAnimal {
    fn default() -> Self {
        Self::new()
    }
}

impl PilotHandler for MonitorAnimal {
    /// Function called from a thread to handle the Monitor Animal Pilot logic
    fn handle(
        &mut self,
        state: &mut RoktrackState,
        device: &mut Roktrack,
        detections: &mut [Detection],
        _tx: Sender<VisionMgmtCommand>,
        property: RoktrackProperty,
    ) {
        log::debug!("Start MonitorAnimal Handle");
        // Assess and handle system safety
        let system_risk = match assess_system_risk(state, device) {
            Some(SystemRisk::StateOff) | Some(SystemRisk::HighTemp) => Some(base::stop(device)),
            None => None,
        };
        if system_risk.is_some() {
            log::debug!("System Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Check animal exist
        if !detections.is_empty() {
            log::warn!("Animal Detected!!");
            device
                .inner
                .clone()
                .lock()
                .unwrap()
                .speak("animal_detecting");
            // Get now.
            let utc = chrono::Utc::now();
            if self.last_detected_time + 60000 < utc.timestamp_millis() as u64 {
                log::debug!("Interval time has elapsed. Re-detection is notified.");
                self.last_detected_time = utc.timestamp_millis() as u64;
                let msg = format!(
                    "{:?} detected.",
                    AnimalClasses::from_u32(detections.first().unwrap().cls)
                        .expect("Unknown animal.")
                );
                let _ = send_line_notify_with_image(&msg, &property.path.img.last, property.conf);
            }
        }
        log::debug!("End MonitorAnimal Handle");
    }
}

/// System Risks
///
#[derive(Debug, Clone)]
enum SystemRisk {
    StateOff,
    HighTemp,
}
/// Identify system-related risks
///
fn assess_system_risk(state: &RoktrackState, device: &Roktrack) -> Option<SystemRisk> {
    if !state.state {
        Some(SystemRisk::StateOff)
    } else if state.pi_temp > 70.0 {
        device.inner.clone().lock().unwrap().speak("high_temp");
        Some(SystemRisk::HighTemp)
    } else {
        None
    }
}
