//! Fill Drive Pilot
//!

// # Normal flow of act phase
//
// Proceed * n
//    |
// ReachMarker
//    |
// TurnKeep
//    |
// TurnMarkerInvisible * n
//    |
// TurnMarkerFound
//    |
// Proceed * n
//    |
// Stand  <- The marker lost for some reason. Stop on the spot, increase the resolution and try to detect it again.
//    |
// StartTurn * n
//    |
// TurnMarkerInvisible * 10
//    |
// TurnCountExceeded  <- The process is stopped because it was not found after the specified number of turns.
//
// # General flow
//
// Start
//   | (CCW laps)
// InvertPhase
//   | (CW laps)
// MissionComplete

use std::sync::mpsc::Sender;

use crate::module::{
    device::motor::Motor,
    device::Roktrack,
    pilot::base,
    pilot::{Phase, RoktrackState},
    util::init::RoktrackProperty,
    vision::detector::{sort, Detection, FilterClass, RoktrackClasses},
    vision::VisionMgmtCommand,
};

use super::{base::select_marker, PilotHandler};

#[derive(Clone, Copy)]
pub struct Fill {}

impl Fill {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self::new()
    }
}

impl PilotHandler for Fill {
    fn handle(
        &mut self,
        state: &mut RoktrackState,
        device: &mut Roktrack,
        detections: &mut [Detection],
        tx: Sender<VisionMgmtCommand>,
        property: RoktrackProperty,
    ) {
        log::debug!("Start Fill Handle");
        // Assess and handle system safety
        let system_risk = match assess_system_risk(state, device) {
            Some(SystemRisk::StateOff) => Some(base::stop(device)),
            Some(SystemRisk::HighTemp) => {
                let res = base::stop(device);
                device.inner.clone().lock().unwrap().speak("high_temp");
                Some(res)
            }
            Some(SystemRisk::Bumped) => {
                let res = base::escape(state, device);
                device.inner.clone().lock().unwrap().speak("bumped");
                Some(res)
            }
            None => None,
        };
        if system_risk.is_some() {
            log::warn!("System Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Assess and handle vision safety
        let vision_risk = match assess_vision_risk(detections) {
            Some(VisionRisk::PersonDetected) => {
                let res = base::stop(device);
                device
                    .inner
                    .clone()
                    .lock()
                    .unwrap()
                    .speak("person_detecting");
                Some(res)
            }
            Some(VisionRisk::RoktrackDetected) => Some(base::stop(device)),
            None => None,
        };
        if vision_risk.is_some() {
            log::warn!("Vision Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Sort markers based on the current phase
        let detections = match state.phase {
            Phase::CCW => sort::right(detections),
            Phase::CW => sort::left(detections),
        };

        // Get the first detected marker or a default one
        let marker = select_marker(property, state, detections, device);
        log::info!("Marker Selected: {:?}", marker);

        // Turn on the work motor
        device.inner.clone().lock().unwrap().work_motor.cw();

        // Calculate constants based on marker and image height
        state.constant = base::calc_constant(state.constant, state.img_height, marker.h);

        let action = assess_situation(state, &marker);
        log::info!("Action is {:?}", action);

        // Handle the current phase
        let _ = match action {
            Some(ActPhase::TurnCountExceeded) => base::halt(state, device, tx),
            Some(ActPhase::TurnMarkerInvisible) => base::reset_ex_height(state, device),
            Some(ActPhase::TurnMarkerFound) => base::set_new_target(state, device, marker),
            Some(ActPhase::InvertPhase) => base::invert_phase(state, device),
            Some(ActPhase::MissionComplete) => base::mission_complete(state, device),
            Some(ActPhase::TurnKeep) => base::keep_turn(state, device, tx),
            Some(ActPhase::Stand) => base::stand(state, tx),
            Some(ActPhase::StartTurn) => base::start_turn(state, device),
            Some(ActPhase::ReachMarker) => base::reach_marker(state, device, marker),
            Some(ActPhase::Proceed) => base::proceed(state, device, marker, tx),
            None => Ok(()),
        };
        log::debug!("End Fill Handle");
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
        Some(SystemRisk::HighTemp)
    } else if device.inner.clone().lock().unwrap().bumper.switch.is_low() {
        Some(SystemRisk::Bumped)
    } else {
        None
    }
}
/// Vision-related risks
///
#[derive(Debug, Clone)]
enum VisionRisk {
    PersonDetected,
    RoktrackDetected,
}
/// Identify vision-related risks
///
fn assess_vision_risk(dets: &mut [Detection]) -> Option<VisionRisk> {
    if !RoktrackClasses::filter(dets, RoktrackClasses::PERSON.to_u32()).is_empty() {
        Some(VisionRisk::PersonDetected)
    } else if !RoktrackClasses::filter(dets, RoktrackClasses::ROKTRACK.to_u32()).is_empty() {
        Some(VisionRisk::RoktrackDetected)
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
    InvertPhase,
    MissionComplete,
    TurnKeep,
    Stand,
    StartTurn,
    ReachMarker,
    Proceed,
}
/// Function to assess the current situation and determine the appropriate action phase
fn assess_situation(state: &RoktrackState, marker: &Detection) -> Option<ActPhase> {
    if 10 <= state.turn_count {
        Some(ActPhase::TurnCountExceeded)
    } else if 0 < state.turn_count {
        if marker.h == 0 {
            Some(ActPhase::TurnMarkerInvisible)
        } else if (marker.h as f32) < state.ex_height as f32 - state.img_height as f32 * 0.015 {
            if state.rest < 0.0 {
                match state.phase {
                    super::Phase::CW => Some(ActPhase::MissionComplete),
                    super::Phase::CCW => Some(ActPhase::InvertPhase),
                }
            } else {
                Some(ActPhase::TurnMarkerFound)
            }
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
