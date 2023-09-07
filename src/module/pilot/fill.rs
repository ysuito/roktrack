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
    vision::detector::{sort, Detection, FilterClass, RoktrackClasses},
    vision::VisionMgmtCommand,
};

/// Function called from a thread to handle the Fill Drive Pilot logic
pub fn handler(
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

    // Assess and handle vision safety
    let vision_risk = match assess_vision_risk(detections) {
        VisionRisk::PersonDetected | VisionRisk::RoktrackDetected => Some(base::stop(device)),
        VisionRisk::None => None,
    };
    if vision_risk.is_some() {
        return; // Risk exists, continue
    }

    // Sort markers based on the current phase
    let detections = match state.phase {
        Phase::CCW => sort::right(detections),
        Phase::CW => sort::left(detections),
    };

    // Get the first detected marker or a default one
    let marker = detections.first().cloned().unwrap_or_default();

    // Turn on the work motor
    device.inner.clone().lock().unwrap().work_motor.cw();

    // Calculate constants based on marker and image height
    state.constant = base::calc_constant(state.constant, state.img_height, marker.h);

    // Handle the current phase
    match assess_situation(state, &marker) {
        ActPhase::TurnCountExceeded => base::halt(state, device),
        ActPhase::TurnMarkerInvisible => base::reset_ex_height(state, device),
        ActPhase::TurnMarkerFound => base::set_new_target(state, device, marker),
        ActPhase::InvertPhase => base::invert_phase(state, device),
        ActPhase::MissionComplete => base::mission_complete(state, device),
        ActPhase::TurnKeep => base::keep_turn(state, device, tx),
        ActPhase::Stand => base::stand(state, tx),
        ActPhase::StartTurn => base::start_turn(state, device),
        ActPhase::ReachMarker => base::reach_marker(state, device, marker),
        ActPhase::Proceed => base::proceed(state, device, marker, tx),
        ActPhase::None => None,
    };
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
/// Vision-related risks
///
#[derive(Debug, Clone)]
enum VisionRisk {
    PersonDetected,
    RoktrackDetected,
    None,
}
/// Identify vision-related risks
///
fn assess_vision_risk(dets: &mut [Detection]) -> VisionRisk {
    if !RoktrackClasses::filter(dets, RoktrackClasses::PERSON).is_empty() {
        VisionRisk::PersonDetected
    } else if !RoktrackClasses::filter(dets, RoktrackClasses::ROKTRACK).is_empty() {
        VisionRisk::RoktrackDetected
    } else {
        VisionRisk::None
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
    None,
}
/// Function to assess the current situation and determine the appropriate action phase
fn assess_situation(state: &RoktrackState, marker: &Detection) -> ActPhase {
    if 10 <= state.turn_count {
        ActPhase::TurnCountExceeded
    } else if 0 < state.turn_count {
        if marker.h == 0 {
            ActPhase::TurnMarkerInvisible
        } else if (marker.h as f32) < state.ex_height as f32 - state.img_height as f32 * 0.015 {
            if state.rest < 0.0 {
                match state.phase {
                    super::Phase::CW => ActPhase::MissionComplete,
                    super::Phase::CCW => ActPhase::InvertPhase,
                }
            } else {
                ActPhase::TurnMarkerFound
            }
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
