//! Provides a loop for autonomous driving.

use crate::module::com::{BleBroadCast, Neighbor, ParentMsg};
use crate::module::pilot::{fill, oneway, Modes, RoktrackState};
use crate::module::util::init::RoktrackProperty;
use crate::module::vision::detector::Detection;
use crate::module::vision::{RoktrackVision, VisionMgmtCommand};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::device::DeviceMgmtCommand;

/// Start the autonomous driving thread.
pub fn run(property: RoktrackProperty) -> JoinHandle<()> {
    // Prepare communication channels for threads.
    // For Vision
    let (channel_vision_mgmt_tx, channel_vision_mgmt_rx): (
        Sender<VisionMgmtCommand>,
        Receiver<VisionMgmtCommand>,
    ) = mpsc::channel();
    let (channel_detections_tx, channel_detections_rx): (
        Sender<Vec<Detection>>,
        Receiver<Vec<Detection>>,
    ) = mpsc::channel();
    // For BLE Communication
    let (channel_neighbor_tx, channel_neighbor_rx): (Sender<Neighbor>, Receiver<Neighbor>) =
        mpsc::channel();
    // For Device Thread (not used in this code)
    let (_channel_device_mgmt_tx, _channel_device_mgmt_rx): (
        Sender<DeviceMgmtCommand>,
        Receiver<DeviceMgmtCommand>,
    ) = mpsc::channel();

    // Initialize the neighbors table.
    let mut neighbors = HashMap::new();

    // Start the BLE communication thread.
    let com = BleBroadCast::new();
    let _com_handler = com.listen(channel_neighbor_tx);

    // Initialize the device (not used in this code).
    let mut device = crate::module::device::Roktrack::new(property.conf.clone());
    // Initialize the vision module and start the inference thread.
    let vision = RoktrackVision::new(property.clone());
    vision.run(channel_detections_tx, channel_vision_mgmt_rx);

    // Initialize the state.
    let mut state = RoktrackState::new();

    thread::spawn(move || loop {
        // Get new neighbor information.
        let neighbor = match channel_neighbor_rx.try_recv() {
            Ok(neighbor) => {
                // Handle commands from the parent (smartphone app).
                if neighbor.identifier == 0 {
                    command_handler(&mut state, &neighbor);
                }
                // Update the neighbor table.
                neighbors.insert(neighbor.identifier, neighbor.clone());
                Some(neighbor)
            }
            Err(_) => None,
        };

        // Get new inference results.
        let detections = match channel_detections_rx.try_recv() {
            Ok(detections) => Some(detections),
            Err(_) => None,
        };

        // If there is no change in the situation, skip the rest of the loop.
        if neighbor.is_none() && detections.is_none() {
            continue;
        }

        // Handle different driving modes.
        match state.mode {
            Modes::Fill => {
                fill::handler(
                    &mut state,
                    &mut device,
                    &mut detections.unwrap(),
                    channel_vision_mgmt_tx.clone(),
                );
            }
            Modes::OneWay => {
                oneway::handler();
            }
            _ => (),
        }

        // Broadcast the state to neighbors.
        let payload = state.dump(&neighbors.clone());
        com.inner
            .clone()
            .lock()
            .unwrap()
            .cast(&state.identifier, payload);

        // Sleep to control the loop rate.
        thread::sleep(Duration::from_millis(100));
    })
}

/// Handle commands received from neighbors.
fn command_handler(state: &mut RoktrackState, neighbor: &Neighbor) {
    if neighbor.dest == 255 {
        match ParentMsg::from_u8(neighbor.msg) {
            // Switch the state if states differ between self and parent.
            ParentMsg::Off => {
                if state.state {
                    state.state = false;
                }
            }
            ParentMsg::On => {
                if !state.state {
                    state.state = true;
                }
            }
            // Reset the state if the current state is off and receives a reset order from the parent.
            ParentMsg::Reset => {
                if !state.state {
                    state.reset();
                }
            }
            // Switch mode
            ParentMsg::Fill => {
                state.mode = Modes::Fill;
            }
            ParentMsg::Oneway => {
                state.mode = Modes::OneWay;
            }
            ParentMsg::Climb => (),
            ParentMsg::Around => (),
            ParentMsg::MonitorPerson => (),
            ParentMsg::MonitorAnimal => (),
            ParentMsg::RoundTrip => (),
            ParentMsg::FollowPerson => (),
            // Manual Control
            ParentMsg::Stop => (),
            ParentMsg::Forward => (),
            ParentMsg::Backward => (),
            ParentMsg::Left => (),
            ParentMsg::Right => (),
            // Others
            _ => (),
        }
    }
}
