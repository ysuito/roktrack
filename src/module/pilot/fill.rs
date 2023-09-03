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

use crate::module::device::motor::Motor;
use crate::module::device::Roktrack;
use crate::module::pilot::{Phase, RoktrackState};
use crate::module::util::init::RoktrackProperty;
use crate::module::vision::detector::onnx::SessionType;
use crate::module::vision::detector::{FilterClass, RoktrackClasses};
use crate::module::vision::RoktrackVision;
use crate::module::{
    pilot::base,
    vision::detector::{sort, Detection},
};

/// Function called from thread
///
pub fn handler(
    prop: &RoktrackProperty,
    state: &mut RoktrackState,
    vision: &RoktrackVision,
    device: &mut Roktrack,
) {
    // take image
    vision.cam.take();

    // System Safety
    let system_risk = assess_system_risk(state, device);
    let system_risk = match system_risk {
        SystemRisk::StateOff => base::stop(device),
        SystemRisk::HighTemp => base::stop(device),
        SystemRisk::Bumped => base::escape(state, device),
        SystemRisk::None => None,
    };
    if system_risk.is_some() {
        return; // risk exists, continue
    }

    // inference
    let mut dets = vision.det.infer(
        &prop.path.img.last.clone(),
        match state.img_width {
            320 => SessionType::Sz320,
            640 => SessionType::Sz640,
            _ => panic!("Invalid Image Size"),
        },
    );
    // Vision Safety
    let vision_risk: VisionRisk = assess_vision_risk(&mut dets);
    let vision_risk = match vision_risk {
        VisionRisk::PersonDetected => base::stop(device),
        VisionRisk::RoktrackDetected => base::escape(state, device),
        VisionRisk::None => None,
    };
    if vision_risk.is_some() {
        return; // risk exists, continue
    }

    // sort marker
    dets = match state.phase {
        Phase::CCW => sort::right(&mut dets),
        Phase::CW => sort::left(&mut dets),
    };
    // pick up a marker
    let marker = match dets.is_empty() {
        true => Detection::default(),
        false => dets[0],
    };

    // work motor on
    device.work_motor.cw();
    // calc const
    state.constant = base::calc_constant(state.constant, state.img_height, marker.h);

    // handle normal
    let act_phase = assess_situation(state, marker);
    match act_phase {
        ActPhase::TurnCountExceeded => base::halt(state, device),
        ActPhase::TurnMarkerInvisible => base::reset_ex_height(state, device),
        ActPhase::TurnMarkerFound => base::set_new_target(state, device, marker),
        ActPhase::InvertPhase => base::invert_phase(state, device),
        ActPhase::MissionComplete => base::mission_complete(state, device),
        ActPhase::TurnKeep => base::keep_turn(state, device),
        ActPhase::Stand => base::stand(state),
        ActPhase::StartTurn => base::start_turn(state, device),
        ActPhase::ReachMarker => base::reach_marker(state, device, marker),
        ActPhase::Proceed => base::proceed(state, device, marker),
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
    } else if device.measure_temp() > 70.0 {
        SystemRisk::HighTemp
    } else if device.bumper.switch.is_low() {
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
/// Identify actions
///
fn assess_situation(state: &RoktrackState, marker: Detection) -> ActPhase {
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
