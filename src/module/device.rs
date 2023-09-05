//! Provides Device Control functionality.
//!
//! This module includes various components for controlling hardware devices, such as motors and speakers.

pub mod base;
pub mod motor;
pub mod speaker;

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{sync::mpsc::Receiver, thread::JoinHandle, time::Duration};

use crate::module::device::motor::Motor;
use crate::module::util::conf::Config;

// File path to get the temperature of the SoC of Raspberry Pi.
const TEMPERATURE_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Device management commands.
pub enum DeviceMgmtCommand {
    Stop,
}

/// Device set.
pub struct Roktrack {
    pub inner: Arc<Mutex<RoktrackInner>>,
}

impl Roktrack {
    /// Creates a new Roktrack device with the given configuration.
    pub fn new(conf: Config) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RoktrackInner::new(conf))),
        }
    }

    /// Runs the device management thread.
    pub fn run(&self, rx: Receiver<DeviceMgmtCommand>) -> JoinHandle<()> {
        let local_self = self.inner.clone();
        thread::spawn(move || {
            loop {
                // Handle Stop command.
                if let Ok(DeviceMgmtCommand::Stop) = rx.try_recv() {
                    local_self.lock().unwrap().stop();
                    continue;
                }
                // Operation Management
                {
                    let utc = chrono::Utc::now();
                    let now = utc.timestamp_millis() as u64;
                    // When the target time is reached, the operation is paused.
                    if now > local_self.clone().lock().unwrap().target_time {
                        local_self.clone().lock().unwrap().pause();
                    }
                }
                // Bumper Interupt
                {
                    if local_self.clone().lock().unwrap().bumper.switch.is_low() {
                        local_self.clone().lock().unwrap().pause();
                    }
                }
                // Sleep to control the loop rate.
                thread::sleep(Duration::from_millis(10));
            }
        })
    }
}

/// Device set containing hardware components.
pub struct RoktrackInner {
    pub drive_motor_right: motor::DriveMotor,
    pub drive_motor_left: motor::DriveMotor,
    pub work_motor: motor::WorkMotor,
    pub bumper: base::Bumper,
    pub turn_adj: f32,    // Turn time adjustment factor
    pub target_time: u64, // Milliseconds
}

impl RoktrackInner {
    /// Creates a new RoktrackInner instance with the given configuration.
    pub fn new(conf: Config) -> Self {
        Self {
            drive_motor_right: motor::DriveMotor::new(
                conf.pin.right_pin1,
                conf.pin.right_pin2,
                conf.pwm.pwm_power_right,
            ),
            drive_motor_left: motor::DriveMotor::new(
                conf.pin.left_pin1,
                conf.pin.left_pin2,
                conf.pwm.pwm_power_left,
            ),
            work_motor: motor::WorkMotor::new(conf.pin.work1_pin, conf.pin.work_ctrl_positive),
            bumper: base::Bumper::new(conf.pin.bumper_pin),
            turn_adj: conf.drive.turn_adj,
            target_time: 0, // Milliseconds
        }
    }

    /// Plays audio files stored in the asset/audio/ folder.
    pub fn speak(&self, name: &str) {
        speaker::speak(name);
    }

    /// Measures the temperature of the Raspberry Pi's SoC.
    pub fn measure_temp(&self) -> f32 {
        let mut f = File::open(TEMPERATURE_FILE).unwrap();
        let mut c = String::new();
        f.read_to_string(&mut c).unwrap();

        // Convert temperature format (e.g., 45678 -> 45.678)
        let temp = format!("{}.{}", &c[0..2], &c[2..5]);
        temp.parse::<f32>().unwrap()
    }

    /// Adjusts the output power of the left and right motors to maintain straightness.
    pub fn adjust_power(&mut self, left: f64, right: f64) {
        let new_left = self.drive_motor_left.power + left;
        if 0.4 < new_left && new_left < 1.0 {
            self.drive_motor_left.power = new_left;
        }
        let new_right = self.drive_motor_right.power + right;
        if 0.4 < new_right && new_right < 1.0 {
            self.drive_motor_right.power = new_right;
        }
    }
}

/// Defines drive system operations.
pub trait Chassis {
    fn set_target_time(&mut self, duration: u64);
    fn stop(&mut self);
    fn pause(&mut self);
    fn forward(&mut self, duration: u64);
    fn backward(&mut self, duration: u64);
    fn left(&mut self, duration: u64);
    fn right(&mut self, duration: u64);
}

impl Chassis for RoktrackInner {
    /// Set the target time for motor control based on the duration.
    fn set_target_time(&mut self, duration: u64) {
        let utc = chrono::Utc::now();
        self.target_time = if duration == 0 {
            utc.timestamp_millis() as u64 + 60000 // 1 minutes
        } else {
            utc.timestamp_millis() as u64 + (duration as f32 * self.turn_adj) as u64
        };
    }

    /// Stop all motors, including the work motor.
    fn stop(&mut self) {
        self.drive_motor_left.stop();
        self.drive_motor_right.stop();
        self.work_motor.stop();
    }

    /// Pause drive motors (left and right).
    fn pause(&mut self) {
        self.drive_motor_left.stop();
        self.drive_motor_right.stop();
    }

    /// Move the machine forward for the specified duration.
    fn forward(&mut self, milsec: u64) {
        self.drive_motor_left.cw();
        self.drive_motor_right.cw();
        self.set_target_time(milsec);
    }

    /// Move the machine backward for the specified duration.
    fn backward(&mut self, milsec: u64) {
        self.drive_motor_left.ccw();
        self.drive_motor_right.ccw();
        self.set_target_time(milsec);
    }

    /// Move the machine left for the specified duration.
    fn left(&mut self, milsec: u64) {
        self.drive_motor_left.ccw();
        self.drive_motor_right.cw();
        self.set_target_time(milsec);
    }

    /// Move the machine right for the specified duration.
    fn right(&mut self, milsec: u64) {
        self.drive_motor_left.cw();
        self.drive_motor_right.ccw();
        self.set_target_time(milsec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    /// Test the drive system.
    #[test]
    fn drive_test() {
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        let roktrack = Roktrack::new(conf);
        println!("device test forward ever");
        roktrack.inner.clone().lock().unwrap().forward(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test backward ever");
        roktrack.inner.clone().lock().unwrap().backward(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test left ever");
        roktrack.inner.clone().lock().unwrap().left(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test right ever");
        roktrack.inner.clone().lock().unwrap().right(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test forward 1sec");
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test backward 1sec");
        roktrack.inner.clone().lock().unwrap().backward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test left 1sec");
        roktrack.inner.clone().lock().unwrap().left(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test right 1sec");
        roktrack.inner.clone().lock().unwrap().right(1000);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test forward again");
        roktrack.inner.clone().lock().unwrap().forward(0);
        println!("device test pause");
        roktrack.inner.clone().lock().unwrap().pause();
        thread::sleep(time::Duration::from_millis(1000));
        println!("device test adjust power 0.9");
        roktrack
            .inner
            .clone()
            .lock()
            .unwrap()
            .adjust_power(-0.1, -0.1);
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test adjust power 0.8");
        roktrack
            .inner
            .clone()
            .lock()
            .unwrap()
            .adjust_power(-0.1, -0.1);
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test adjust power 0.7");
        roktrack
            .inner
            .clone()
            .lock()
            .unwrap()
            .adjust_power(-0.1, -0.1);
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test adjust power 0.6");
        roktrack
            .inner
            .clone()
            .lock()
            .unwrap()
            .adjust_power(-0.1, -0.1);
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        println!("device test adjust power 0.5");
        roktrack
            .inner
            .clone()
            .lock()
            .unwrap()
            .adjust_power(-0.1, -0.1);
        roktrack.inner.clone().lock().unwrap().forward(1000);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.inner.clone().lock().unwrap().stop();
        println!("device test done!");
    }

    /// Test temperature measurement.
    #[test]
    fn measure_temp_test() {
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        let roktrack = Roktrack::new(conf);
        assert!(roktrack.inner.clone().lock().unwrap().measure_temp() < 20.0);
        assert!(roktrack.inner.clone().lock().unwrap().measure_temp() < 70.0);
    }
}
