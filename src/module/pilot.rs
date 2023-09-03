//! Provide automatic operation modes.
//!
pub mod base;
pub mod fill;
pub mod oneway;

use super::com::Neighbor;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

/// Automatic operation modes.
///
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

/// Automatic operation modes's methods.
///
impl Modes {
    /// Convert int to operation mode.
    ///
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
    /// convert operation mode to int.
    ///
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

/// Direction of laps.
///
#[derive(Debug, Clone)]
pub enum Phase {
    CW,
    CCW,
}

/// State for auto pilot.
///
#[derive(Debug, Clone)]
pub struct RoktrackState {
    pub state: bool,        // on / off
    pub mode: Modes,        // drive mode
    pub turn_count: i8,     // continuous turn counter
    pub ex_height: u16,     // last seen marker height for searching next one
    pub rest: f32,          // remaining work. 0.0 -> 1.0
    pub target_height: u16, // when you approach this target height, start looking for the next marker.
    pub phase: Phase,       // direction of laps.
    pub constant: f32,      // amount to be subtracted from rest for each marker approach.
    pub marker_id: i8,      // record the ID assigned to the marker when in ocr mode is on
    pub pi_temp: f32,       // rpi's soc temp
    pub msg: u8,            // current state message
    pub identifier: u8,     // my identifier
    pub img_width: u32,     // width of image to process
    pub img_height: u32,    // height of image to process
}

/// RoktrackState's default methods.
///
impl Default for RoktrackState {
    /// Return RoktrackState default.
    ///
    fn default() -> Self {
        Self::new()
    }
}

/// RoktrackState's methods.
///
impl RoktrackState {
    /// RoktrackState's constructor.
    ///
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
            marker_id: -1,
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

    /// Reset RoktrackState
    ///
    pub fn reset(&mut self) {
        self.state = false;
        self.turn_count = -1;
        self.ex_height = 0;
        self.rest = 1.0;
        self.target_height = (240.0 * 0.9) as u16;
        self.phase = Phase::CCW;
        self.constant = 0.005;
        self.marker_id = -1;
        self.msg = 255;
        self.img_width = 320;
        self.img_height = 240;
    }

    /// CCW -> CW with resetting counters.
    ///
    pub fn invert_phase(&mut self) {
        self.reset();
        self.phase = Phase::CW;
    }

    /// Dump self state for broadcasting.
    ///
    pub fn dump(&mut self, neighbors: HashMap<u8, Neighbor>) -> Vec<u8> {
        let used_identifiers: Vec<u8> = neighbors.keys().cloned().collect();
        if used_identifiers.clone().contains(&self.identifier) {
            let pool: Vec<u8> = (1..250).filter(|x| !used_identifiers.contains(x)).collect();
            self.identifier = *pool.choose(&mut rand::thread_rng()).unwrap();
        }
        // construct first byte
        let state_and_rest = format!("{:b}{:b}", self.state as u8, (self.rest * 100.0) as u8);
        let state_and_rest: u8 = isize::from_str_radix(&state_and_rest, 2).unwrap() as u8;
        // construct payload
        let mut val = vec![
            state_and_rest,                  // state and rest
            self.pi_temp as u8,              // pi temp
            Modes::to_u8(self.mode.clone()), // mode as int
            self.msg,                        // msg
            255,                             // dest
        ];
        // padding
        val.resize(23, 0);
        val
    }
}
