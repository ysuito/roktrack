//! Provide Motor Control
//!

use rppal::gpio::Gpio;

/// Basic Motor Function
///
pub trait Motor {
    fn cw(&mut self) {}
    fn ccw(&mut self) {}
    fn stop(&mut self) {}
}
/// Drive Motor
///
pub struct DriveMotor {
    pin1: rppal::gpio::OutputPin,
    pin2: rppal::gpio::OutputPin,
    pub power: f64,
}
/// Drive Motor's methosd
///
impl DriveMotor {
    // Drive Motor's constructor
    pub fn new(pin1: u8, pin2: u8, power: f64) -> Self {
        let gpio1 = Gpio::new().unwrap();
        let gpio2 = Gpio::new().unwrap();

        Self {
            pin1: gpio1.get(pin1).unwrap().into_output(),
            pin2: gpio2.get(pin2).unwrap().into_output(),
            power,
        }
    }
}
/// Implement motor basic functionality for Drive Motor.
///
impl Motor for DriveMotor {
    /// Rotate CW.
    fn cw(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_low();
        self.pin2.set_pwm_frequency(100.0, self.power).unwrap();
    }
    /// Rotate CCW.
    fn ccw(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_pwm_frequency(100.0, self.power).unwrap();
        self.pin2.set_low();
    }
    /// Stop.
    fn stop(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_low();
        self.pin2.set_low();
    }
}
/// Work Motor for cutting grass.
///
pub struct WorkMotor {
    pin1: rppal::gpio::OutputPin,
    positive_relay: bool,
}
/// Work Motor's methods.
///
impl WorkMotor {
    pub fn new(pin1: u8, positive_relay: bool) -> Self {
        let gpio1 = Gpio::new().unwrap();

        Self {
            pin1: gpio1.get(pin1).unwrap().into_output(),
            positive_relay,
        }
    }
}
/// Implement motor basic functionality for Work Motor.
///
impl Motor for WorkMotor {
    /// Rotate CW.
    ///
    fn cw(&mut self) {
        if self.positive_relay {
            self.pin1.set_high();
        } else {
            self.pin1.set_low();
        }
    }
    /// CCW is not supported, as it is handled by a relay, not a motor driver.
    ///
    fn ccw(&mut self) {
        panic!("Work Motor ccw Not Implemented.")
    }
    /// Stop.
    ///
    fn stop(&mut self) {
        if self.positive_relay {
            self.pin1.set_low();
        } else {
            self.pin1.set_high();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn drive_motor_test() {
        // left motor test
        let mut dml = DriveMotor::new(22, 23, 1.0);
        // left cw
        println!("left motor cw test power:{}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.power = 0.7;
        println!("left motor cw test power:{}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.power = 0.5;
        println!("left motor cw test power:{}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.stop();
        // left ccw
        dml.power = 1.0;
        println!("left motor ccw test power:{}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.power = 0.7;
        println!("left motor ccw test power:{}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.power = 0.5;
        println!("left motor ccw test power:{}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dml.stop();
        // right motor test
        let mut dmr = DriveMotor::new(24, 25, 1.0);
        // right cw
        println!("right motor cw test power:{}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.power = 0.7;
        println!("right motor cw test power:{}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.power = 0.5;
        println!("right motor cw test power:{}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.stop();
        // right ccw
        dmr.power = 1.0;
        println!("right motor ccw test power:{}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.power = 0.7;
        println!("right motor ccw test power:{}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.power = 0.5;
        println!("right motor ccw test power:{}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));
        dmr.stop();
    }
    #[test]
    fn work_motor_test() {
        let mut wm = WorkMotor::new(14, false);
        println!("work motor cw test");
        wm.cw();
        thread::sleep(time::Duration::from_millis(2000));
        println!("work motor stop test");
        wm.stop();
        thread::sleep(time::Duration::from_millis(5000));
    }
}
