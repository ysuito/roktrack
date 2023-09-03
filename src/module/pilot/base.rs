//! Common Drive Functions
//!

use crate::module::com::ChildMsg;
use crate::module::device::Chassis;
use crate::module::device::Roktrack;
use crate::module::pilot::RoktrackState;
use crate::module::vision::detector::Detection;

use super::Phase;

/// Stop drive and work motor.
///
pub fn stop(device: &mut Roktrack) -> Option<()> {
    device.stop();
    Some(())
}

/// Escape action. Back -> Turn -> Forward -> Counter Trun
///
pub fn escape(state: &RoktrackState, device: &mut Roktrack) -> Option<()> {
    device.backward(2000);
    match state.phase {
        Phase::CCW => device.left(500),
        Phase::CW => device.right(500),
    };
    device.forward(2000);
    match state.phase {
        Phase::CCW => device.right(500),
        Phase::CW => device.left(500),
    };
    Some(())
}
/// Terminate driving.
///
pub fn halt(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    state.state = false;
    state.msg = ChildMsg::to_u8(ChildMsg::TargetNotFound);
    device.stop();
    device.speak("cone_not_found");
    Some(())
}
/// Increase Image Resolution
///
pub fn upscale(state: &mut RoktrackState) {
    let new_width: f32 = 640.0;
    let new_height: f32 = 480.0;
    let ratio = new_width / state.img_width as f32;
    state.img_height = new_height as u32;
    state.img_width = new_width as u32;
    state.ex_height = (state.ex_height as f32 * ratio) as u16;
    state.target_height = (state.target_height as f32 * ratio) as u16;
}
/// Decrease Image Resolution
///
pub fn downscale(state: &mut RoktrackState) {
    let new_width: f32 = 320.0;
    let new_height: f32 = 240.0;
    let ratio = new_width / state.img_width as f32;
    state.img_height = new_height as u32;
    state.img_width = new_width as u32;
    state.ex_height = (state.ex_height as f32 * ratio) as u16;
    state.target_height = (state.target_height as f32 * ratio) as u16;
}

/// Reset ex height.
/// If roktrack start to turn and the marker is no longer visible, reset the ex-height
/// and set the target to the next marker to be found, regardless of its height.
///
pub fn reset_ex_height(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    state.msg = ChildMsg::to_u8(ChildMsg::TargetLost);
    state.ex_height = (state.img_height as f32 * 1.1) as u16;
    match state.phase {
        Phase::CCW => device.left(500),
        Phase::CW => device.right(500),
    };
    state.turn_count += 1;
    Some(())
}

/// Sets the amount of rest subtraction when reaching a marker.
/// Set only if the constant is not set. In other words, when a marker is recognized for the first time,
/// the constant is calculated from marker size and set.
///
pub fn calc_constant(cur_constant: f32, cam_height: u32, marker_height: u32) -> f32 {
    if cur_constant == 0.0 {
        let marker_share = marker_height as f32 / cam_height as f32;
        if 0.1 * marker_share < 0.005 {
            0.1 * marker_share
        } else {
            0.005
        }
    } else {
        cur_constant
    }
}
/// Start the lap in the opposite direction.
///
pub fn invert_phase(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    state.invert_phase();
    device.pause();
    Some(())
}
/// Work targets are achieved and the system is shut down.
///
pub fn mission_complete(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    state.state = false;
    device.stop();
    Some(())
}
/// Keep turning for searching next marker.
///
pub fn keep_turn(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    match state.phase {
        Phase::CCW => device.left(500),
        Phase::CW => device.right(500),
    };
    if state.turn_count > 4 {
        upscale(state);
    }
    state.turn_count += 1;
    Some(())
}
/// Found new marker. Calc target_height to next one.
/// Reset turn_count. Subtract rest.
///
pub fn set_new_target(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    marker: Detection,
) -> Option<()> {
    state.msg = ChildMsg::to_u8(ChildMsg::NewTargetFound);
    device.speak("new_cone_found");
    state.rest -= state.constant;
    state.target_height = (marker.h as f32
        + (state.img_height as f32 * 0.9 - marker.h as f32) * (state.rest.powf(2.0)))
        as u16;
    state.turn_count = 0;
    Some(())
}
/// The marker could be lost and the detection could be attempted again with higher resolution on the spot.
///
pub fn stand(state: &mut RoktrackState) -> Option<()> {
    upscale(state);
    state.msg = ChildMsg::to_u8(ChildMsg::TargetLost);
    state.turn_count = 0;
    Some(())
}
/// Start turning. Search any next marker.
///
pub fn start_turn(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    match state.phase {
        Phase::CCW => device.left(500),
        Phase::CW => device.right(500),
    };
    state.turn_count = 1;
    state.ex_height = (state.img_height as f32 * 1.1) as u16;
    state.target_height = 0;
    Some(())
}
/// Reach marker. marker_height > target_height
/// Start turn.
///
pub fn reach_marker(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    marker: Detection,
) -> Option<()> {
    device.pause();
    state.turn_count = 1;
    state.ex_height = marker.h as u16;
    state.target_height = 0;
    state.msg = ChildMsg::to_u8(ChildMsg::ReachTarget);
    device.speak("close_to_cone");
    match state.phase {
        Phase::CCW => device.left(500),
        Phase::CW => device.right(500),
    };
    Some(())
}
/// Calculate the difference in the target direction.
/// Returns the difference as a ratio of pixel widths.
///
/// # Example
/// ## Simple
/// --------------------
/// |         |        |
/// |         |        |
/// |         |   A    |
/// |         |<-->    |
/// |         | 32     |
/// --------------------
///          320
/// => return -0.1
///
/// --------------------
/// |    A    |        |
/// |    <--->|        |
/// |     64  |        |
/// |         |        |
/// |         |        |
/// --------------------
///          320
/// => return 0.2
///
/// ## Near Marker
/// If the target is closer than a certain distance to the marker, shift the target in the lap phase direction.
/// --------------------
/// |         |        |
/// |      |  O        | 2
/// |   144| OOO       | 4
/// |      |OOOOO      | 0
/// |         |-->|64  |
/// --------------------
///          320
/// => return -0.2
///
fn get_diff(
    marker_center_x: f32,
    marker_height: u32,
    cam_height: u32,
    cam_width: u32,
    phase: Phase,
) -> f32 {
    let mut offset = if marker_height as f32 > cam_height as f32 * 0.5 {
        cam_width as f32 / 2.0 * 0.4
    } else {
        0.0
    };
    offset = match phase {
        Phase::CCW => offset,
        Phase::CW => -offset,
    };
    (cam_width as f32 / 2.0 - marker_center_x + offset) / cam_width as f32
}
/// Proceed to target marker.
/// Calculate the difference between the target direction and the direction of travel.
/// If the difference is large based on the results, the output of the left and right motors is
/// adjusted and turn to counter direction. If the difference is small, the output of the left and right
/// motors is adjusted and the machine moves forward.
///
pub fn proceed(state: &mut RoktrackState, device: &mut Roktrack, marker: Detection) -> Option<()> {
    // calc difference
    let diff = get_diff(
        marker.xc,
        marker.h,
        state.img_height,
        state.img_width,
        state.phase.clone(),
    );
    let val = (0.1 * diff).abs() as f64;
    if 0.15 < diff {
        device.left(100);
        device.adjust_power(-val, val);
    } else if 0.03 < diff {
        device.adjust_power(-val, val);
        device.forward(0);
    } else if diff < -0.15 {
        device.right(100);
        device.adjust_power(val, -val);
    } else if diff < -0.03 {
        device.adjust_power(val, -val);
        device.forward(0);
    } else {
        device.forward(0);
    }
    // check no need for high resolution processing
    if marker.h as f32 > state.img_height as f32 * 0.05 && state.img_width == 640 {
        downscale(state);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_constant_test() {
        assert_eq!(
            format!("{:.3}", calc_constant(0.0, 240, 120)),
            0.005.to_string()
        );
        assert_eq!(
            format!("{:.3}", calc_constant(0.0, 240, 12)),
            0.005.to_string()
        );
        assert_eq!(
            format!("{:.4}", calc_constant(0.0, 240, 6)),
            0.0025.to_string()
        );
        assert_eq!(
            format!("{:.4}", calc_constant(0.0, 240, 6)),
            0.0025.to_string()
        );
        assert_eq!(
            format!("{:.3}", calc_constant(0.005, 240, 6)),
            0.005.to_string()
        );
    }

    #[test]
    fn scale_test() {
        let mut state = RoktrackState::new();
        state.ex_height = 100;
        assert_eq!(state.ex_height, 100);
        assert_eq!(state.target_height, 216);
        assert_eq!(state.img_height, 240);
        assert_eq!(state.img_width, 320);
        upscale(&mut state);
        assert_eq!(state.ex_height, 200);
        assert_eq!(state.target_height, 432);
        assert_eq!(state.img_height, 480);
        assert_eq!(state.img_width, 640);
        downscale(&mut state);
        assert_eq!(state.ex_height, 100);
        assert_eq!(state.target_height, 216);
        assert_eq!(state.img_height, 240);
        assert_eq!(state.img_width, 320);
    }
}
