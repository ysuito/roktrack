//! Provides a loop for autonomous driving.

use crate::module::com::{BleBroadCast, Neighbor, ParentMsg};
use crate::module::pilot::{Modes, RoktrackState};
use crate::module::util::init::RoktrackProperty;
use crate::module::vision::detector::Detection;
use crate::module::vision::{RoktrackVision, VisionMgmtCommand};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::device::{Chassis, DeviceMgmtCommand, Roktrack};
use super::pilot::fill::Fill;
use super::pilot::follow_person::FollowPerson;
use super::pilot::monitor_animal::MonitorAnimal;
use super::pilot::monitor_person::MonitorPerson;
use super::pilot::oneway::OneWay;
use super::pilot::PilotHandler;

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
    let (_channel_device_mgmt_tx, channel_device_mgmt_rx): (
        Sender<DeviceMgmtCommand>,
        Receiver<DeviceMgmtCommand>,
    ) = mpsc::channel();

    // Initialize the neighbors table.
    let mut neighbors = HashMap::new();

    // Start the BLE communication thread.
    let com = BleBroadCast::new();
    let _com_handler = com.listen(channel_neighbor_tx);

    // Start the device thread.
    let mut device = crate::module::device::Roktrack::new(property.conf.clone());
    device.run(channel_device_mgmt_rx);

    // Initialize the vision module and start the inference thread.
    let vision = RoktrackVision::new(property.clone());
    vision.run(channel_detections_tx, channel_vision_mgmt_rx);

    // Initialize the state.
    let mut state = RoktrackState::new();
    // Initialize drive handler.
    let mut handler: Box<dyn PilotHandler> = Box::new(Fill::new());

    thread::spawn(move || loop {
        // Sleep to control the loop rate.
        thread::sleep(Duration::from_millis(10));

        // Get new neighbor information.
        if let Ok(neighbor) = channel_neighbor_rx.try_recv() {
            // Update the neighbor table.
            neighbors.insert(neighbor.identifier, neighbor.clone());
            // Check command
            if let Some(n) = command_handler(
                &mut state,
                &neighbor,
                &mut device,
                channel_vision_mgmt_tx.clone(),
            ) {
                // If there are new instructions, replace the handler.
                handler = n;
            }
        }

        // Get new inference results.
        let detections = match channel_detections_rx.try_recv() {
            Ok(detections) => Some(detections),
            Err(_) => None,
        };

        // If there is no detections, skip the rest of the loop.
        if detections.is_none() {
            continue;
        }

        // handle driving
        handler.handle(
            &mut state,
            &mut device,
            &mut detections.unwrap(),
            channel_vision_mgmt_tx.clone(),
            property.clone(),
        );

        // Broadcast my state to neighbors.
        let payload = state.dump(&neighbors.clone());
        com.inner
            .clone()
            .lock()
            .unwrap()
            .cast(&state.identifier, payload);
    })
}

/// Handle commands received from neighbors.
fn command_handler(
    state: &mut RoktrackState,
    neighbor: &Neighbor,
    device: &mut Roktrack,
    tx: Sender<VisionMgmtCommand>,
) -> Option<Box<dyn PilotHandler>> {
    // Handle commands from the parent (smartphone app).
    if neighbor.identifier == 0 && neighbor.dest == 255 {
        match ParentMsg::from_u8(neighbor.msg) {
            // Switch the state if states differ between new state and old state.
            ParentMsg::Off => {
                if state.state {
                    state.state = false;
                    device.inner.clone().lock().unwrap().stop();
                    tx.send(VisionMgmtCommand::Off).unwrap();
                }
                None
            }
            ParentMsg::On => {
                if !state.state {
                    state.state = true;
                    tx.send(VisionMgmtCommand::On).unwrap();
                }
                None
            }
            // Reset the state if the current state is off and receives a reset order from the parent.
            ParentMsg::Reset => {
                if !state.state {
                    state.reset();
                }
                None
            }
            // Switch mode
            ParentMsg::Fill => {
                if !state.state && state.mode != Modes::Fill {
                    state.mode = Modes::Fill;
                    tx.send(VisionMgmtCommand::SwitchSessionPylon).unwrap();
                    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
                    Some(Box::new(Fill::new()))
                } else {
                    None
                }
            }
            ParentMsg::Oneway => {
                if !state.state && state.mode != Modes::OneWay {
                    state.mode = Modes::OneWay;
                    tx.send(VisionMgmtCommand::SwitchSessionPylon).unwrap();
                    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
                    Some(Box::new(OneWay::new()))
                } else {
                    None
                }
            }
            ParentMsg::Climb => None,
            ParentMsg::Around => None,
            ParentMsg::MonitorPerson => {
                if !state.state && state.mode != Modes::MonitorPerson {
                    state.mode = Modes::MonitorPerson;
                    tx.send(VisionMgmtCommand::SwitchSessionPylon).unwrap();
                    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
                    Some(Box::new(MonitorPerson::new()))
                } else {
                    None
                }
            }
            ParentMsg::MonitorAnimal => {
                if !state.state && state.mode != Modes::MonitorAnimal {
                    state.mode = Modes::MonitorAnimal;
                    tx.send(VisionMgmtCommand::SwitchSessionAnimal).unwrap();
                    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
                    Some(Box::new(MonitorAnimal::new()))
                } else {
                    None
                }
            }
            ParentMsg::RoundTrip => None,
            ParentMsg::FollowPerson => {
                if !state.state && state.mode != Modes::FollowPerson {
                    state.mode = Modes::FollowPerson;
                    tx.send(VisionMgmtCommand::SwitchSessionPylon).unwrap();
                    tx.send(VisionMgmtCommand::SwitchSz320).unwrap();
                    Some(Box::new(FollowPerson::new()))
                } else {
                    None
                }
            }
            // Manual Control
            ParentMsg::Stop => None,
            ParentMsg::Forward => None,
            ParentMsg::Backward => None,
            ParentMsg::Left => None,
            ParentMsg::Right => None,
            // Others
            _ => None,
        }
    } else {
        None
    }
}
