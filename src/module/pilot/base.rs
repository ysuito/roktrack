//! Common Drive Functions
//!

use std::sync::mpsc::Sender;
use std::thread;
use std::time;

use crate::module::com::ChildMsg;
use crate::module::device::Chassis;
use crate::module::device::Roktrack;
use crate::module::pilot::RoktrackState;
use crate::module::vision::detector::Detection;
use crate::module::vision::VisionMgmtCommand;

use super::Phase;

/// Stop the drive and work motor.
///
/// This function stops both the drive and the work motor of the Roktrack.
///
/// # Arguments
///
/// * `device` - A mutable reference to the Roktrack device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn stop(device: &mut Roktrack) -> Option<()> {
    device.inner.clone().lock().unwrap().stop();
    Some(())
}

/// Perform an escape action to recover from an obstacle or risk.
///
/// This function instructs the Roktrack to perform an escape action, which typically involves
/// moving backward, turning, moving forward, and then turning again in the opposite direction.
/// The specific actions depend on the current phase of the pilot (CW or CCW).
///
/// # Arguments
///
/// * `state` - A reference to the RoktrackState representing the current state of the pilot.
/// * `device` - A mutable reference to the Roktrack device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn escape(state: &RoktrackState, device: &mut Roktrack) -> Option<()> {
    let binding = device.inner.clone();
    let mut device_lock = binding.lock().unwrap();
    device_lock.backward(2000);
    thread::sleep(time::Duration::from_millis(2000));
    match state.phase {
        Phase::CCW => device_lock.left(500),
        Phase::CW => device_lock.right(500),
    };
    thread::sleep(time::Duration::from_millis(500));
    device_lock.forward(2000);
    thread::sleep(time::Duration::from_millis(2000));
    match state.phase {
        Phase::CCW => device_lock.right(500),
        Phase::CW => device_lock.left(500),
    };
    thread::sleep(time::Duration::from_millis(500));
    Some(())
}

/// Terminate the driving and set the state to off.
///
/// This function stops the Roktrack, sets the state to off, and sends a message indicating
/// that the target was not found.
///
/// # Arguments
///
/// * `state` - A mutable reference to the RoktrackState representing the current state of the pilot.
/// * `device` - A mutable reference to the Roktrack device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn halt(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    state.state = false;
    state.msg = ChildMsg::to_u8(ChildMsg::TargetNotFound);
    device.inner.clone().lock().unwrap().stop();
    device.inner.clone().lock().unwrap().speak("cone_not_found");
    Some(())
}

/// Increase the image resolution and adjust state.
///
/// This function sends a command to the vision system to upscale the image resolution.
/// It also updates the local state to reflect the new image dimensions and adjusts
/// the expected and target heights accordingly.
///
/// # Arguments
///
/// * `state` - A mutable reference to the RoktrackState representing the current state of the pilot.
/// * `tx` - A sender for sending commands to the vision management system.
pub fn upscale(state: &mut RoktrackState, tx: Sender<VisionMgmtCommand>) {
    // Command vision to upscale
    tx.send(VisionMgmtCommand::SwitchSz640).unwrap();
    // Change local state
    let new_width: f32 = 640.0;
    let new_height: f32 = 480.0;
    let ratio = new_width / state.img_width as f32;
    state.img_height = new_height as u32;
    state.img_width = new_width as u32;
    state.ex_height = (state.ex_height as f32 * ratio) as u16;
    state.target_height = (state.target_height as f32 * ratio) as u16;
}

/// Decrease the image resolution and adjust state.
///
/// This function sends a command to the vision system to downscale the image resolution.
/// It also updates the local state to reflect the new image dimensions and adjusts
/// the expected and target heights accordingly.
///
/// # Arguments
///
/// * `state` - A mutable reference to the RoktrackState representing the current state of the pilot.
/// * `tx` - A sender for sending commands to the vision management system.
pub fn downscale(state: &mut RoktrackState, tx: Sender<VisionMgmtCommand>) {
    // Command vision to downscale
    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
    // Change local state
    let new_width: f32 = 320.0;
    let new_height: f32 = 240.0;
    let ratio = new_width / state.img_width as f32;
    state.img_height = new_height as u32;
    state.img_width = new_width as u32;
    state.ex_height = (state.ex_height as f32 * ratio) as u16;
    state.target_height = (state.target_height as f32 * ratio) as u16;
}

/// Reset the last seen height (`ex_height`) and turn direction when the marker is lost.
///
/// If Roktrack starts to turn and the marker is no longer visible, this function resets the
/// expected height to 110% of the image height (`img_height`) and sets the target to the next marker
/// to be found, regardless of its height.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn reset_ex_height(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    // Notify that the target is lost
    state.msg = ChildMsg::to_u8(ChildMsg::TargetLost);
    // Reset the expected height to 110% of the image height
    state.ex_height = (state.img_height as f32 * 1.1) as u16;
    // Adjust the turn direction based on the current phase
    match state.phase {
        Phase::CCW => device.inner.clone().lock().unwrap().left(500),
        Phase::CW => device.inner.clone().lock().unwrap().right(500),
    };
    // Increment the turn count
    state.turn_count += 1;
    Some(())
}

/// Calculate and set the constant for height adjustments when reaching a marker.
///
/// This function sets the constant value for height adjustments based on the current constant value
/// and the ratio of the marker height to the camera height. If the current constant is not set (0.0),
/// it calculates a new constant based on the marker size and sets it.
///
/// # Arguments
///
/// * `cur_constant` - The current constant value.
/// * `cam_height` - The height of the camera image.
/// * `marker_height` - The height of the marker detected.
///
/// # Returns
///
/// The calculated constant value.
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

/// Start laps in the opposite direction (invert the phase).
///
/// This function inverts the current lap phase (e.g., from CCW to CW) and pauses the Roktrack's movement.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn invert_phase(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    // Invert the current phase (lap direction)
    state.invert_phase();
    // Pause the Roktrack's movement
    device.inner.clone().lock().unwrap().pause();
    Some(())
}

/// Perform actions when mission targets are achieved, and the system is shut down.
///
/// This function sets the pilot's state to false (off) and stops the Roktrack's movement.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn mission_complete(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    // Set the pilot's state to false (off)
    state.state = false;
    // Stop the Roktrack's movement
    device.inner.clone().lock().unwrap().stop();
    Some(())
}

/// Keep turning to search for the next marker.
///
/// This function instructs the Roktrack to continue turning to search for the next marker.
/// It also checks the turn count, and if it exceeds a threshold, it requests an image resolution upscale.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
/// * `tx` - A sender for sending commands to the vision management system.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn keep_turn(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    tx: Sender<VisionMgmtCommand>,
) -> Option<()> {
    // Instruct the Roktrack to turn based on the current phase
    match state.phase {
        Phase::CCW => device.inner.clone().lock().unwrap().left(500),
        Phase::CW => device.inner.clone().lock().unwrap().right(500),
    };
    // If the turn count exceeds 4, request an image resolution upscale
    if state.turn_count > 4 {
        upscale(state, tx);
    }
    // Increment the turn count
    state.turn_count += 1;
    Some(())
}

/// Set a new target based on the detected marker.
///
/// This function sets a new target height for the Roktrack to reach based on the properties of the
/// detected marker. It also resets the turn count and subtracts the rest value from the target height.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
/// * `marker` - The `Detection` structure representing the detected marker.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn set_new_target(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    marker: Detection,
) -> Option<()> {
    // Notify that a new target is found
    state.msg = ChildMsg::to_u8(ChildMsg::NewTargetFound);
    // Speak a notification
    device.inner.clone().lock().unwrap().speak("new_cone_found");
    // Subtract the rest value
    state.rest -= state.constant;
    // Calculate the new target height based on the marker properties
    state.target_height = (marker.h as f32
        + (state.img_height as f32 * 0.9 - marker.h as f32) * (state.rest.powf(2.0)))
        as u16;
    // Reset the turn count
    state.turn_count = 0;
    Some(())
}

/// Transition to higher resolution to reattempt marker detection.
///
/// This function transitions to higher resolution, sends a "target lost" message, and resets the turn count.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `tx` - A sender for sending commands to the vision management system.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn stand(state: &mut RoktrackState, tx: Sender<VisionMgmtCommand>) -> Option<()> {
    // Transition to higher resolution
    upscale(state, tx);
    // Send "target lost" message
    state.msg = ChildMsg::to_u8(ChildMsg::TargetLost);
    // Reset the turn count
    state.turn_count = 0;
    Some(())
}

/// Start turning to search for the next marker.
///
/// This function starts the Roktrack's movement in the specified direction, initializes the turn count,
/// and sets the expected height to 110% of the image height while clearing the target height.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn start_turn(state: &mut RoktrackState, device: &mut Roktrack) -> Option<()> {
    // Start the Roktrack's movement in the specified direction
    match state.phase {
        Phase::CCW => device.inner.clone().lock().unwrap().left(500),
        Phase::CW => device.inner.clone().lock().unwrap().right(500),
    };
    // Initialize the turn count
    state.turn_count = 1;
    // Set the expected height to 110% of the image height
    state.ex_height = (state.img_height as f32 * 1.1) as u16;
    // Clear the target height
    state.target_height = 0;
    Some(())
}

/// Reach a marker with marker height greater than the target height.
/// Start the next turn to search for the next marker.
///
/// This function pauses the Roktrack's movement, resets the turn count to 1, sets the expected height
/// to the marker's height, clears the target height, sends a "reach target" message, and speaks a notification.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
/// * `marker` - The `Detection` structure representing the detected marker.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn reach_marker(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    marker: Detection,
) -> Option<()> {
    // Pause the Roktrack's movement
    device.inner.clone().lock().unwrap().pause();
    // Reset the turn count to 1
    state.turn_count = 1;
    // Set the expected height to the marker's height
    state.ex_height = marker.h as u16;
    // Clear the target height
    state.target_height = 0;
    // Send "reach target" message
    state.msg = ChildMsg::to_u8(ChildMsg::ReachTarget);
    // Speak a "close to cone" notification
    device.inner.clone().lock().unwrap().speak("close_to_cone");
    // Start the next turn in the specified direction
    match state.phase {
        Phase::CCW => device.inner.clone().lock().unwrap().left(500),
        Phase::CW => device.inner.clone().lock().unwrap().right(500),
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

/// Proceed to the target marker.
///
/// This function calculates the difference between the target direction and the current direction of travel
/// based on the marker's position and dimensions. It adjusts the left and right motors' output to align with
/// the target direction. If the difference is large, it initiates a turn in the counter-direction; if it's
/// small, the machine moves forward. It also checks whether high-resolution processing is required and
/// sends the corresponding command to the vision system.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `RoktrackState` representing the current state of the pilot.
/// * `device` - A mutable reference to the `Roktrack` device.
/// * `marker` - The `Detection` structure representing the detected marker.
/// * `tx` - A sender for sending commands to the vision management system.
///
/// # Returns
///
/// An `Option<()>` where `Some(())` indicates success.
pub fn proceed(
    state: &mut RoktrackState,
    device: &mut Roktrack,
    marker: Detection,
    tx: Sender<VisionMgmtCommand>,
) -> Option<()> {
    // Calculate the difference between the target direction and the current direction of travel
    let diff = get_diff(
        marker.xc,
        marker.h,
        state.img_height,
        state.img_width,
        state.phase.clone(),
    );

    // Calculate a value based on the difference for motor adjustments
    let val = (0.1 * diff).abs() as f64;

    // Adjust motor outputs based on the difference
    if 0.15 < diff {
        device.inner.clone().lock().unwrap().left(100);
        device.inner.clone().lock().unwrap().adjust_power(-val, val);
    } else if 0.03 < diff {
        device.inner.clone().lock().unwrap().adjust_power(-val, val);
        device.inner.clone().lock().unwrap().forward(0);
    } else if diff < -0.15 {
        device.inner.clone().lock().unwrap().right(100);
        device.inner.clone().lock().unwrap().adjust_power(val, -val);
    } else if diff < -0.03 {
        device.inner.clone().lock().unwrap().adjust_power(val, -val);
        device.inner.clone().lock().unwrap().forward(0);
    } else {
        device.inner.clone().lock().unwrap().forward(0);
    }

    // Check if high-resolution processing is needed based on marker height and current image resolution
    if marker.h as f32 > state.img_height as f32 * 0.05 && state.img_width == 640 {
        // Send a command to downscale the resolution
        downscale(state, tx);
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, Receiver};

    // Import the functions and types being tested
    use super::*;

    #[test]
    fn calc_constant_test() {
        // Test cases for the `calc_constant` function
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
        // Create channels for testing vision management commands
        let (channel_vision_mgmt_tx, _channel_vision_mgmt_rx): (
            Sender<VisionMgmtCommand>,
            Receiver<VisionMgmtCommand>,
        ) = mpsc::channel();

        // Initialize a test state
        let mut state = RoktrackState::new();
        state.ex_height = 100;

        // Test initial state values
        assert_eq!(state.ex_height, 100);
        assert_eq!(state.target_height, 216);
        assert_eq!(state.img_height, 240);
        assert_eq!(state.img_width, 320);

        // Test upscaling
        upscale(&mut state, channel_vision_mgmt_tx.clone());
        assert_eq!(state.ex_height, 200);
        assert_eq!(state.target_height, 432);
        assert_eq!(state.img_height, 480);
        assert_eq!(state.img_width, 640);

        // Test downscaling
        downscale(&mut state, channel_vision_mgmt_tx.clone());
        assert_eq!(state.ex_height, 100);
        assert_eq!(state.target_height, 216);
        assert_eq!(state.img_height, 240);
        assert_eq!(state.img_width, 320);
    }
}
