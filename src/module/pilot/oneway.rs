//! OneWay Drive Pilot

use std::sync::mpsc::Sender;

use super::PilotHandler;
use crate::module::{
    device::motor::Motor,
    device::Roktrack,
    pilot::base,
    pilot::{Phase, RoktrackState},
    util::init::RoktrackProperty,
    vision::detector::{sort, Detection, FilterClass, RoktrackClasses},
    vision::VisionMgmtCommand,
};

pub struct OneWay {}

impl OneWay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for OneWay {
    fn default() -> Self {
        Self::new()
    }
}

impl PilotHandler for OneWay {
    /// Function called from a thread to handle the OneWay Drive Pilot logic
    fn handle(
        &mut self,
        state: &mut RoktrackState,
        device: &mut Roktrack,
        detections: &mut [Detection],
        tx: Sender<VisionMgmtCommand>,
        _property: RoktrackProperty,
    ) {
        log::debug!("Start OneWay Handle");
        // Assess and handle system safety
        let system_risk = match assess_system_risk(state, device) {
            SystemRisk::StateOff | SystemRisk::HighTemp => Some(base::stop(device)),
            SystemRisk::Bumped => Some(base::escape(state, device)),
            SystemRisk::None => None,
        };
        if system_risk.is_some() {
            log::debug!("System Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Assess and handle vision safety
        let vision_risk = match assess_vision_risk(detections) {
            VisionRisk::PersonDetected | VisionRisk::RoktrackDetected => Some(base::stop(device)),
            VisionRisk::None => None,
        };
        if vision_risk.is_some() {
            log::debug!("Vision Risk Exists. Continue.");
            return; // Risk exists, continue
        }

        // Sort markers based on the current phase
        let detections = match state.turn_count {
            1 => sort::small(detections),
            _ => match state.phase {
                Phase::CCW => sort::right(detections),
                Phase::CW => sort::left(detections),
            },
        };

        // Get the first detected marker or a default one
        let marker = detections.first().cloned().unwrap_or_default();
        log::debug!("Marker Selected: {:?}", marker);

        // Turn on the work motor
        device.inner.clone().lock().unwrap().work_motor.cw();

        let action = assess_situation(state, &marker);
        log::debug!("Action is {:?}", action);

        // Handle the current phase
        match action {
            ActPhase::TurnCountExceeded => base::halt(state, device),
            ActPhase::TurnMarkerInvisible => base::reset_ex_height(state, device),
            ActPhase::TurnMarkerFound => base::set_new_target(state, device, marker),
            ActPhase::TurnKeep => base::keep_turn(state, device, tx),
            ActPhase::Stand => base::stand(state, tx),
            ActPhase::StartTurn => base::start_turn(state, device),
            ActPhase::ReachMarker => base::reach_marker(state, device, marker),
            ActPhase::Proceed => base::proceed(state, device, marker, tx),
            ActPhase::None => None,
        };
        log::debug!("End OneWay Handle");
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
    if !RoktrackClasses::filter(dets, RoktrackClasses::PERSON.to_u32()).is_empty() {
        VisionRisk::PersonDetected
    } else if !RoktrackClasses::filter(dets, RoktrackClasses::ROKTRACK.to_u32()).is_empty() {
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
