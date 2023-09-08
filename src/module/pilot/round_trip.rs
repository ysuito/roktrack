//! Roundtrip Pilot between marker and person.

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::Roktrack,
    pilot::base,
    pilot::RoktrackState,
    vision::detector::{sort, Detection, FilterClass, RoktrackClasses},
    vision::VisionMgmtCommand,
};

pub struct RoundTrip {
    target_object: RoundTripObject,
}

impl RoundTrip {
    pub fn new() -> Self {
        Self {
            target_object: RoundTripObject::Marker,
        }
    }
}

impl Default for RoundTrip {
    fn default() -> Self {
        Self::new()
    }
}

impl PilotHandler for RoundTrip {
    /// Function called from a thread to handle the OneWay Drive Pilot logic
    fn handle(
        &mut self,
        state: &mut RoktrackState,
        device: &mut Roktrack,
        detections: &mut [Detection],
        tx: Sender<VisionMgmtCommand>,
    ) {
        // Assess and handle system safety
        let system_risk = match assess_system_risk(state, device) {
            SystemRisk::StateOff | SystemRisk::HighTemp => Some(base::stop(device)),
            SystemRisk::Bumped => Some(base::escape(state, device)),
            SystemRisk::None => None,
        };
        if system_risk.is_some() {
            return; // Risk exists, continue
        }

        // Sort markers based on the current phase
        let detections = sort::big(detections);
        let detections = match self.target_object {
            RoundTripObject::Marker => {
                RoktrackClasses::filter(&mut detections.clone(), RoktrackClasses::PYLON)
            }
            RoundTripObject::Person => {
                RoktrackClasses::filter(&mut detections.clone(), RoktrackClasses::PERSON)
            }
        };

        // Get the first detected marker or a default one
        let marker = detections.first().cloned().unwrap_or_default();

        // Handle the current phase
        match assess_situation(state, &marker) {
            ActPhase::TurnCountExceeded => base::halt(state, device),
            ActPhase::TurnMarkerInvisible => base::reset_ex_height(state, device),
            ActPhase::TurnMarkerFound => base::set_new_target(state, device, marker),
            ActPhase::TurnKeep => base::keep_turn(state, device, tx),
            ActPhase::Stand => base::stand(state, tx),
            ActPhase::StartTurn => base::start_turn(state, device),
            ActPhase::ReachMarker => {
                self.target_object = match self.target_object {
                    RoundTripObject::Marker => RoundTripObject::Person,
                    RoundTripObject::Person => RoundTripObject::Marker,
                };
                base::reach_marker(state, device, marker)
            }
            ActPhase::Proceed => base::proceed(state, device, marker, tx),
            ActPhase::None => None,
        };
    }
}

//// Target Object
#[derive(Debug, Clone)]
pub enum RoundTripObject {
    Marker,
    Person,
}

impl RoundTripObject {
    /// Convert RoundTripObject to RoktrackClasses
    pub fn to_cls(target: RoundTripObject) -> RoktrackClasses {
        match target {
            RoundTripObject::Marker => RoktrackClasses::PYLON,
            RoundTripObject::Person => RoktrackClasses::PERSON,
        }
    }
}

/// System Risks
///
#[derive(Debug, Clone)]
enum SystemRisk {
    StateOff,
    HighTemp,
    Bumped,
    None,
}
/// Identify system-related risks
///
fn assess_system_risk(state: &RoktrackState, device: &Roktrack) -> SystemRisk {
    if !state.state {
        SystemRisk::StateOff
    } else if device.inner.clone().lock().unwrap().measure_temp() > 70.0 {
        SystemRisk::HighTemp
    } else if device.inner.clone().lock().unwrap().bumper.switch.is_low() {
        SystemRisk::Bumped
    } else {
        SystemRisk::None
    }
}
/// Actions for Fill Drive Pilot
///
#[derive(Debug, Clone)]
enum ActPhase {
    TurnCountExceeded,
    TurnMarkerInvisible,
    TurnMarkerFound,
    TurnKeep,
    Stand,
    StartTurn,
    ReachMarker,
    Proceed,
    None,
}
/// Function to assess the current situation and determine the appropriate action phase
fn assess_situation(state: &RoktrackState, marker: &Detection) -> ActPhase {
    if 7 <= state.turn_count {
        ActPhase::TurnCountExceeded
    } else if 0 < state.turn_count {
        if marker.h == 0 {
            ActPhase::TurnMarkerInvisible
        } else if (marker.h as f32) < state.ex_height as f32 - state.img_height as f32 * 0.015 {
            ActPhase::TurnMarkerFound
        } else {
            ActPhase::TurnKeep
        }
    } else if marker.h == 0 {
        if state.turn_count == -1 {
            ActPhase::Stand
        } else if state.turn_count == 0 {
            ActPhase::StartTurn
        } else {
            ActPhase::None
        }
    } else if state.target_height <= marker.h as u16 {
        ActPhase::ReachMarker
    } else {
        ActPhase::Proceed
    }
}
