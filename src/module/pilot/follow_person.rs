//! Follow Person Pilot
//!

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::Chassis,
    device::Roktrack,
    pilot::base,
    pilot::RoktrackState,
    vision::detector::{sort, Detection, FilterClass, RoktrackClasses},
    vision::VisionMgmtCommand,
};

pub struct FollowPerson {}

impl FollowPerson {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FollowPerson {
    fn default() -> Self {
        Self::new()
    }
}

impl PilotHandler for FollowPerson {
    /// Function called from a thread to handle the Follow Person Pilot logic
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
        let detections = RoktrackClasses::filter(&mut detections.clone(), RoktrackClasses::PERSON);

        // Get the first detected marker or a default one
        let marker = detections.first().cloned().unwrap_or_default();

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
            ActPhase::ReachMarker => {
                device.inner.lock().unwrap().pause();
                Some(())
            }
            ActPhase::Proceed => base::proceed(state, device, marker, tx),
            ActPhase::None => None,
        };
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
