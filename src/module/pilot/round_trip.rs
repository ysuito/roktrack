//! Roundtrip Pilot between marker and person.

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::Roktrack,
    pilot::base,
    pilot::RoktrackState,
    util::init::RoktrackProperty,
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
        _property: RoktrackProperty,
    ) {
        log::debug!("Start RoundTrip Handle");
        // Assess and handle system safety
        let system_risk = match assess_system_risk(state, device) {
            Some(SystemRisk::StateOff) | Some(SystemRisk::HighTemp) => Some(base::stop(device)),
            Some(SystemRisk::Bumped) => Some(base::escape(state, device)),
            None => None,
        };
        if system_risk.is_some() {
            log::debug!("System Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Sort markers based on the current target object
        let detections = sort::big(detections);
        let detections = match self.target_object {
            RoundTripObject::Marker => {
                RoktrackClasses::filter(&mut detections.clone(), (RoktrackClasses::PYLON).to_u32())
            }
            RoundTripObject::Person => {
                RoktrackClasses::filter(&mut detections.clone(), (RoktrackClasses::PERSON).to_u32())
            }
        };

        // Get the first detected marker or a default one
        let marker = detections.first().cloned().unwrap_or_default();
        log::debug!("Marker Selected: {:?}", marker);

        let action = assess_situation(state, &marker);
        log::debug!("Action is {:?}", action);

        // Handle the current phase
        let _ = match action {
            Some(ActPhase::TurnCountExceeded) => base::halt(state, device, tx),
            Some(ActPhase::TurnMarkerInvisible) => base::reset_ex_height(state, device),
            Some(ActPhase::TurnMarkerFound) => base::set_new_target(state, device, marker),
            Some(ActPhase::TurnKeep) => base::keep_turn(state, device, tx),
            Some(ActPhase::Stand) => base::stand(state, tx),
            Some(ActPhase::StartTurn) => base::start_turn(state, device),
            Some(ActPhase::ReachMarker) => {
                self.target_object = match self.target_object {
                    RoundTripObject::Marker => {
                        log::debug!("Target Object Switch. Marker -> Person");
                        RoundTripObject::Person
                    }
                    RoundTripObject::Person => {
                        log::debug!("Target Object Switch. Person -> Marker");
                        RoundTripObject::Marker
                    }
                };
                base::reach_marker(state, device, marker)
            }
            Some(ActPhase::Proceed) => base::proceed(state, device, marker, tx),
            None => Ok(()),
        };
        log::debug!("End RoundTrip Handle");
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
}
/// Identify system-related risks
///
fn assess_system_risk(state: &RoktrackState, device: &Roktrack) -> Option<SystemRisk> {
    if !state.state {
        Some(SystemRisk::StateOff)
    } else if state.pi_temp > 70.0 {
        device.inner.clone().lock().unwrap().speak("high_temp");
        Some(SystemRisk::HighTemp)
    } else if device.inner.clone().lock().unwrap().bumper.switch.is_low() {
        device.inner.clone().lock().unwrap().speak("bumped");
        Some(SystemRisk::Bumped)
    } else {
        None
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
}
/// Function to assess the current situation and determine the appropriate action phase
fn assess_situation(state: &RoktrackState, marker: &Detection) -> Option<ActPhase> {
    if 7 <= state.turn_count {
        Some(ActPhase::TurnCountExceeded)
    } else if 0 < state.turn_count {
        if marker.h == 0 {
            Some(ActPhase::TurnMarkerInvisible)
        } else if (marker.h as f32) < state.ex_height as f32 - state.img_height as f32 * 0.015 {
            Some(ActPhase::TurnMarkerFound)
        } else {
            Some(ActPhase::TurnKeep)
        }
    } else if marker.h == 0 {
        if state.turn_count == -1 {
            Some(ActPhase::Stand)
        } else if state.turn_count == 0 {
            Some(ActPhase::StartTurn)
        } else {
            None
        }
    } else if state.target_height <= marker.h as u16 {
        Some(ActPhase::ReachMarker)
    } else {
        Some(ActPhase::Proceed)
    }
}
