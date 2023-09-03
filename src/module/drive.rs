//! Provide Loop for Drive.
//!

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time};

use crate::module::pilot::Modes;
use crate::module::pilot::{fill, oneway};

use super::com::{Neighbor, ParentMsg};
use super::pilot::RoktrackState;

/// Start drive thread
///
pub fn run(
    neighbor_table: Arc<Mutex<HashMap<u8, crate::module::com::Neighbor>>>,
    property: crate::module::util::init::RoktrackProperty,
    com: crate::module::com::BleBroadCast,
) -> JoinHandle<()> {
    // init device
    let mut device = crate::module::device::Roktrack::new(property.conf.clone());
    // init vision
    let vision = crate::module::vision::RoktrackVision::new(property.clone());
    // init state
    let mut state = crate::module::pilot::RoktrackState::new();
    // neighbor info sharing table
    let table_clone: Arc<Mutex<HashMap<u8, crate::module::com::Neighbor>>> =
        Arc::clone(&neighbor_table);
    thread::spawn(move || loop {
        // read com table if new msg found, do below.
        let neighbors = table_clone.clone().lock().unwrap().clone();

        // extract command
        // 0 is indentifier of commander(smartphone app)
        if let Some(v) = neighbors.get(&0) {
            command_handler(&mut state, v);
        }
        // select mode
        match state.mode {
            Modes::Fill => fill::handler(&property, &mut state, &vision, &mut device),
            Modes::OneWay => oneway::handler(),
            _ => (),
        }

        // cast my state
        let payload = state.dump(neighbors);
        com.cast(&state.identifier, payload);

        // loop wait
        thread::sleep(time::Duration::from_millis(100));
    })
}

/// Handle commands from neighbor.
///
fn command_handler(state: &mut RoktrackState, neighbor: &Neighbor) {
    if neighbor.dest == 255 {
        match ParentMsg::from_u8(neighbor.msg) {
            // Switch state if states differ between self and parent.
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
            // Reset state if current state is off and receive reset order from parent.
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
