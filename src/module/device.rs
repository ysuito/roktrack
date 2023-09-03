//! Provide Device Control.
//!
pub mod base;
pub mod motor;
pub mod speaker;

use std::fs::File;
use std::io::Read;
use std::{thread, time};

use crate::module::device::motor::Motor;
use crate::module::util::conf::Config;

// File to get temperature of SoC of raspberry pi.
const TEMPERATURE_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Device aggregator
///
pub struct Roktrack {
    pub drive_motor_right: motor::DriveMotor,
    pub drive_motor_left: motor::DriveMotor,
    pub work_motor: motor::WorkMotor,
    pub bumper: base::Bumper,
    pub turn_adj: f32, // turn time adjustment factor
}

/// Device's methods
///
impl Roktrack {
    /// Roktrack constructor
    ///
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
        }
    }

    /// Play audio files stored in the asset/audio/ folder
    ///
    pub fn speak(&self, name: &str) {
        speaker::speak(name);
    }

    /// Get RPi's SoC temp
    ///
    pub fn measure_temp(&self) -> f32 {
        let mut f = File::open(TEMPERATURE_FILE).unwrap();
        let mut c = String::new();
        f.read_to_string(&mut c).unwrap();

        // 45678 -> 45.678
        let temp = format!("{}.{}", &c[0..2], &c[2..5]);
        temp.parse::<f32>().unwrap()
    }

    /// Adjust the output of the left and right motors. Mainly for straightness.
    ///
    pub fn adjust_power(&mut self, left: f64, right: f64) {
        let new_left = self.drive_motor_left.power + left;
        println!("new left: {}", new_left);
        if 0.4 < new_left && new_left < 1.0 {
            self.drive_motor_left.power = new_left;
        }
        let new_right = self.drive_motor_right.power + right;
        println!("new right: {}", new_right);
        if 0.4 < new_right && new_right < 1.0 {
            self.drive_motor_right.power = new_right;
        }
    }
}

/// Define drive system operation
///
pub trait Chassis {
    fn stop(&mut self);
    fn pause(&mut self);
    fn forward(&mut self, duration: u64);
    fn backward(&mut self, duration: u64);
    fn left(&mut self, duration: u64);
    fn right(&mut self, duration: u64);
}

/// Drive system operation implementation
///
impl Chassis for Roktrack {
    /// Stop all motors including work motor
    ///
    fn stop(&mut self) {
        self.drive_motor_left.stop();
        self.drive_motor_right.stop();
        self.work_motor.stop();
    }
    /// Stop drive motor not including work motor
    ///
    fn pause(&mut self) {
        self.drive_motor_left.stop();
        self.drive_motor_right.stop();
    }
    /// Move the machine forward.
    ///
    /// * `milsec` - Time to move. If 0, it moves forever.
    fn forward(&mut self, milsec: u64) {
        match milsec {
            0 => {
                self.drive_motor_left.cw();
                self.drive_motor_right.cw();
            }
            _ => {
                self.drive_motor_left.cw();
                self.drive_motor_right.cw();
                thread::sleep(time::Duration::from_millis(
                    (milsec as f32 * self.turn_adj) as u64,
                ));
                self.pause();
            }
        }
    }
    /// Move the machine backward.
    ///
    /// * `milsec` - Time to move. If 0, it moves forever.
    fn backward(&mut self, milsec: u64) {
        match milsec {
            0 => {
                self.drive_motor_left.ccw();
                self.drive_motor_right.ccw();
            }
            _ => {
                self.drive_motor_left.ccw();
                self.drive_motor_right.ccw();
                thread::sleep(time::Duration::from_millis(
                    (milsec as f32 * self.turn_adj) as u64,
                ));
                self.pause();
            }
        }
    }
    /// Move the machine left.
    ///
    /// * `milsec` - Time to move. If 0, it moves forever.
    fn left(&mut self, milsec: u64) {
        match milsec {
            0 => {
                self.drive_motor_left.ccw();
                self.drive_motor_right.cw();
            }
            _ => {
                self.drive_motor_left.ccw();
                self.drive_motor_right.cw();
                thread::sleep(time::Duration::from_millis(
                    (milsec as f32 * self.turn_adj) as u64,
                ));
                self.pause();
            }
        }
    }
    /// Move the machine right.
    ///
    /// * `milsec` - Time to move. If 0, it moves forever.
    fn right(&mut self, milsec: u64) {
        match milsec {
            0 => {
                self.drive_motor_left.cw();
                self.drive_motor_right.ccw();
            }
            _ => {
                self.drive_motor_left.cw();
                self.drive_motor_right.ccw();
                thread::sleep(time::Duration::from_millis(
                    (milsec as f32 * self.turn_adj) as u64,
                ));
                self.pause();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn drive_test() {
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        let mut roktrack = Roktrack::new(conf);
        println!("device test forward ever");
        roktrack.forward(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.stop();
        println!("device test backward ever");
        roktrack.backward(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.stop();
        println!("device test left ever");
        roktrack.left(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.stop();
        println!("device test right ever");
        roktrack.right(0);
        thread::sleep(time::Duration::from_millis(2000));
        roktrack.stop();
        println!("device test forward 1sec");
        roktrack.forward(1000);
        println!("device test backward 1sec");
        roktrack.backward(1000);
        println!("device test left 1sec");
        roktrack.left(1000);
        println!("device test right 1sec");
        roktrack.right(1000);
        roktrack.stop();
        println!("device test forward again");
        roktrack.forward(0);
        println!("device test pause");
        roktrack.pause();
        thread::sleep(time::Duration::from_millis(1000));
        println!("device test adjust power 0.9");
        roktrack.adjust_power(-0.1, -0.1);
        roktrack.forward(1000);
        println!("device test adjust power 0.8");
        roktrack.adjust_power(-0.1, -0.1);
        roktrack.forward(1000);
        println!("device test adjust power 0.7");
        roktrack.adjust_power(-0.1, -0.1);
        roktrack.forward(1000);
        println!("device test adjust power 0.6");
        roktrack.adjust_power(-0.1, -0.1);
        roktrack.forward(1000);
        println!("device test adjust power 0.5");
        roktrack.adjust_power(-0.1, -0.1);
        roktrack.forward(1000);
        roktrack.stop();
        println!("device test done!");
    }

    #[test]
    fn measure_temp_test() {
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        let roktrack = Roktrack::new(conf);
        assert!(roktrack.measure_temp() < 20.0);
        assert!(roktrack.measure_temp() < 70.0);
    }
}
