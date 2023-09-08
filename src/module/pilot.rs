//! This module provides automatic operation modes.

// Import the submodules for operation modes
pub mod base; // Base module
pub mod fill; // Fill module
pub mod follow_person; // Follow person module
pub mod monitor_animal; // Monitoring animal module
pub mod monitor_person;
pub mod oneway; // One-way module
pub mod round_trip; // Round-trip between person and marker module // Monitoring person module

use std::sync::mpsc::Sender;

use super::com::Neighbor; // Import the Neighbor type from the com module
use rand::{self, seq::SliceRandom, Rng};
use std::collections::HashMap; // Import HashMap for storage // Import random number generation

use super::{
    device::Roktrack,
    vision::{detector::Detection, VisionMgmtCommand},
}; // Import random number generation

/// Automatic operation modes.
#[derive(Debug, Clone, PartialEq)]
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
}

impl Default for RoktrackState {
    fn default() -> Self {
        Self::new()
    }
}

impl RoktrackState {
    /// Create a new RoktrackState with default values.
    pub fn new() -> Self {
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
            identifier: rand::thread_rng().gen_range(1..250),
            img_width: 320,
            img_height: 240,
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
    }

    /// Invert the phase (CCW -> CW) and reset counters.
    pub fn invert_phase(&mut self) {
        self.reset();
        self.phase = Phase::CW;
    }

    /// Dump the state for broadcasting.
    pub fn dump(&mut self, neighbors: &HashMap<u8, Neighbor>) -> Vec<u8> {
        let used_identifiers: Vec<u8> = neighbors.keys().cloned().collect();
        if used_identifiers.contains(&self.identifier) {
            let pool: Vec<u8> = (1..250).filter(|x| !used_identifiers.contains(x)).collect();
            self.identifier = *pool.choose(&mut rand::thread_rng()).unwrap();
        }
        // Construct the first byte
        let state_and_rest = format!("{:b}{:b}", self.state as u8, (self.rest * 100.0) as u8);
        let state_and_rest: u8 = isize::from_str_radix(&state_and_rest, 2).unwrap() as u8;
        // Construct the payload
        let mut val = vec![
            state_and_rest,                  // State and rest
            self.pi_temp as u8,              // Pi temperature
            Modes::to_u8(self.mode.clone()), // Mode as int
            self.msg,                        // Message
            255,                             // Destination
        ];
        // Padding
        val.resize(23, 0);
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
        let mut state = RoktrackState::new();
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
            state.dump(&neighbors),
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
        detections: &mut [Detection],
        tx: Sender<VisionMgmtCommand>,
    ) {
    }
}
