//! This module provides automatic operation modes.

// Import the submodules for operation modes
pub mod base; // Base module
pub mod fill; // Fill module
pub mod follow_person; // Follow person module
pub mod monitor_animal; // Monitoring animal module
pub mod monitor_person; // Monitoring person module
pub mod oneway; // One-way module
pub mod round_trip; // Round-trip between person and marker module

use super::{
    com::Neighbor, // Import the Neighbor type from the com module
    device::Roktrack,
    util::{conf::Config, init::RoktrackProperty},
    vision::{VisionMgmtCommand, VisualInfo},
};
use rand::{self, seq::SliceRandom}; // Import random number generation
use std::collections::HashMap;
use std::sync::mpsc::Sender; // Import HashMap for storage

/// Automatic operation modes.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Modes {
    Fill,
    OneWay,
    Climb,
    Around,
    MonitorPerson,
    MonitorAnimal,
    RoundTrip,
    FollowPerson,
    Unknown,
}

impl Modes {
    /// Convert an string to an operation mode.
    pub fn from_string(s: &str) -> Modes {
        match s {
            "fill" => Modes::Fill,
            "oneway" => Modes::OneWay,
            "climb" => Modes::Climb,
            "around" => Modes::Around,
            "monitor_animal" => Modes::MonitorAnimal,
            "monitor_person" => Modes::MonitorPerson,
            "round_trip" => Modes::RoundTrip,
            "follow_person" => Modes::FollowPerson,
            _ => Modes::Unknown,
        }
    }

    /// Convert an integer to an operation mode.
    pub fn from_u8(i: u8) -> Modes {
        match i {
            0 => Modes::Fill,
            1 => Modes::OneWay,
            2 => Modes::Climb,
            3 => Modes::Around,
            4 => Modes::MonitorPerson,
            5 => Modes::MonitorAnimal,
            6 => Modes::RoundTrip,
            7 => Modes::FollowPerson,
            _ => Modes::Unknown,
        }
    }

    /// Convert an operation mode to an integer.
    pub fn to_u8(mode: Modes) -> u8 {
        match mode {
            Modes::Fill => 0,
            Modes::OneWay => 1,
            Modes::Climb => 2,
            Modes::Around => 3,
            Modes::MonitorPerson => 4,
            Modes::MonitorAnimal => 5,
            Modes::RoundTrip => 6,
            Modes::FollowPerson => 7,
            _ => 255,
        }
    }
}

/// This enum represents the direction of laps.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    CW,
    CCW,
}

/// This struct represents the state for auto-pilot.
#[derive(Debug, Clone)]
pub struct RoktrackState {
    pub state: bool,           // On / Off
    pub mode: Modes,           // Drive mode
    pub turn_count: i8,        // Continuous turn counter
    pub ex_height: u16,        // Last seen marker height for searching the next one
    pub rest: f32,             // Remaining work (0.0 -> 1.0)
    pub target_height: u16, // When you approach this target height, start looking for the next marker.
    pub phase: Phase,       // Direction of laps
    pub constant: f32,      // Amount to be subtracted from rest for each marker approach
    pub marker_id: Option<u8>, // Record the ID assigned to the marker when OCR mode is on
    pub pi_temp: f32,       // Raspberry Pi's SoC temperature
    pub msg: u8,            // Current state message
    pub identifier: u8,     // My identifier
    pub img_width: u32,     // Width of the image to process
    pub img_height: u32,    // Height of the image to process
    pub diff: f32,          // Normalized marker gap to center.
    pub marker_height: u32, // Normalized marker height.
}

impl RoktrackState {
    /// Create a new RoktrackState with default values.
    pub fn new(conf: Config) -> Self {
        Self {
            state: true,
            mode: Modes::Fill,
            turn_count: -1,
            ex_height: 0,
            rest: 1.0,
            target_height: (240.0 * 0.9) as u16,
            phase: Phase::CCW,
            constant: 0.005,
            marker_id: None,
            pi_temp: 0.0,
            msg: 255,
            // Identifier's Preserved Addresses
            // 0: commander
            // 251-254: preserved
            // 255: broadcast
            identifier: conf.system.identifier,
            img_width: 320,
            img_height: 240,
            diff: 0.0,
            marker_height: 0,
        }
    }

    /// Reset RoktrackState to default values.
    pub fn reset(&mut self) {
        self.state = false;
        self.turn_count = -1;
        self.ex_height = 0;
        self.rest = 1.0;
        self.target_height = (240.0 * 0.9) as u16;
        self.phase = Phase::CCW;
        self.constant = 0.005;
        self.marker_id = None;
        self.msg = 255;
        self.img_width = 320;
        self.img_height = 240;
        self.diff = 0.0;
        self.marker_height = 0;
    }

    /// Invert the phase (CCW -> CW) and reset counters.
    pub fn invert_phase(&mut self) {
        self.reset();
        self.phase = Phase::CW;
    }

    /// Dump the state for broadcasting.
    pub fn dump(
        &mut self,
        neighbors: &HashMap<u8, Neighbor>,
        conf: Config,
        device: &Roktrack,
    ) -> Vec<u8> {
        let used_identifiers: Vec<u8> = neighbors.keys().cloned().collect();
        if used_identifiers.contains(&self.identifier) {
            let pool: Vec<u8> = (1..250).filter(|x| !used_identifiers.contains(x)).collect();
            self.identifier = *pool.choose(&mut rand::thread_rng()).unwrap();
        }
        // Construct the first byte
        let state_and_rest = format!("{:b}{:b}", self.state as u8, (self.rest * 100.0) as u8);
        let state_and_rest: u8 = isize::from_str_radix(&state_and_rest, 2).unwrap_or(0) as u8;
        // u8 variables
        let left_power_u8 =
            (device.inner.clone().lock().unwrap().drive_motor_left.power * 100.0) as u8;
        let right_power_u8 =
            (device.inner.clone().lock().unwrap().drive_motor_left.power * 100.0) as u8;
        let diff_u8 = ((self.diff + 1.0) * 127.0) as u8;
        let marker_height_u8 = (self.marker_height as f32 / self.img_height as f32 * 100.0) as u8;
        // Construct the payload
        let mut val = vec![
            state_and_rest,          // State and rest
            self.pi_temp as u8,      // Pi temperature
            Modes::to_u8(self.mode), // Mode as int
            self.msg,                // Message
            255,                     // Destination
            conf.system.appearance,  // Appearance
            left_power_u8,           // Left Motor Power
            right_power_u8,          // Right Motor Power
            diff_u8,                 // Normalized f32 diff to u8. (-1 ~ 1) -> (0 ~ 255)
            marker_height_u8,        // u8 marker height.
        ];
        // Padding
        val.resize(23, 0);
        log::debug!("Dump My State: {:?}", val);
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_conversion_test() {
        // from u8
        assert_eq!(Modes::from_u8(0), Modes::Fill);
        assert_eq!(Modes::from_u8(254), Modes::Unknown);
        // to u8
        assert_eq!(Modes::to_u8(Modes::Fill), 0);
        assert_eq!(Modes::to_u8(Modes::Unknown), 255);
    }

    #[test]
    fn roktrack_state_test() {
        let property = crate::module::util::init::resource::init();
        let device = Roktrack::new(property.conf.clone());
        let mut state = RoktrackState::new(property.conf.clone());
        // reset test
        state.rest = 0.9;
        assert_eq!(state.rest, 0.9);
        state.reset();
        assert_eq!(state.rest, 1.0);
        // invert phase test
        assert_eq!(state.phase, Phase::CCW);
        state.invert_phase();
        assert_eq!(state.phase, Phase::CW);
        // dump test
        let neighbors = HashMap::new();
        assert_eq!(
            state.dump(&neighbors, property.conf, &device),
            [100, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,]
        )
    }
}

#[allow(unused_variables)]
/// Basement for pilot handler's
pub trait PilotHandler: Send + Sync {
    fn handle(
        &mut self,
        state: &mut RoktrackState,
        device: &mut Roktrack,
        visual_info: &mut VisualInfo,
        tx: Sender<VisionMgmtCommand>,
        property: RoktrackProperty,
    ) {
    }
}
