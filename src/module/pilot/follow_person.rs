//! Follow Person Pilot
//!

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::Chassis,
    device::Roktrack,
    pilot::base,
    pilot::RoktrackState,
    util::init::RoktrackProperty,
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
        _property: RoktrackProperty,
    ) {
        log::debug!("Start FollowPerson Handle");
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

        // Sort markers based on the current phase
        let detections = sort::big(detections);
        let detections =
            RoktrackClasses::filter(&mut detections.clone(), (RoktrackClasses::PERSON).to_u32());

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
            Some(ActPhase::InvertPhase) => base::invert_phase(state, device),
            Some(ActPhase::MissionComplete) => base::mission_complete(state, device),
            Some(ActPhase::TurnKeep) => base::keep_turn(state, device, tx),
            Some(ActPhase::Stand) => base::stand(state, tx),
            Some(ActPhase::StartTurn) => base::start_turn(state, device),
            Some(ActPhase::ReachMarker) => {
                log::debug!("Reach Marker pausing.");
                device.inner.lock().unwrap().pause();
                Ok(())
            }
            Some(ActPhase::Proceed) => base::proceed(state, device, marker, tx),
            None => Ok(()),
        };
        log::debug!("End FollowPerson Handle");
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
