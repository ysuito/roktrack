//! Provides Motor Control functionality.

use rppal::gpio::Gpio;

/// Defines the basic Motor trait.
pub trait Motor {
    /// Rotate the motor clockwise.
    fn cw(&mut self) {}
    /// Rotate the motor counterclockwise.
    fn ccw(&mut self) {}
    /// Stop the motor.
    fn stop(&mut self) {}
}

/// Represents a Drive Motor.
pub struct DriveMotor {
    pin1: rppal::gpio::OutputPin,
    pin2: rppal::gpio::OutputPin,
    pub power: f64,
}

impl DriveMotor {
    /// Creates a new DriveMotor instance.
    ///
    /// # Arguments
    ///
    /// * `pin1` - GPIO pin number for motor control 1.
    /// * `pin2` - GPIO pin number for motor control 2.
    /// * `power` - Motor power (0.0 to 1.0).
    ///
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

impl Motor for DriveMotor {
    /// Rotate the drive motor clockwise (CW).
    fn cw(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_low();
        self.pin2.set_pwm_frequency(100.0, self.power).unwrap();
    }

    /// Rotate the drive motor counterclockwise (CCW).
    fn ccw(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_pwm_frequency(100.0, self.power).unwrap();
        self.pin2.set_low();
    }

    /// Stop the drive motor.
    fn stop(&mut self) {
        self.pin1.clear_pwm().unwrap();
        self.pin2.clear_pwm().unwrap();
        self.pin1.set_low();
        self.pin2.set_low();
    }
}

/// Represents a Work Motor for tasks like cutting grass.
pub struct WorkMotor {
    pin1: rppal::gpio::OutputPin,
    positive_relay: bool,
}

impl WorkMotor {
    /// Creates a new WorkMotor instance.
    ///
    /// # Arguments
    ///
    /// * `pin1` - GPIO pin number for work motor control.
    /// * `positive_relay` - Whether the motor control uses a positive relay (true) or not (false).
    ///
    pub fn new(pin1: u8, positive_relay: bool) -> Self {
        let gpio1 = Gpio::new().unwrap();

        Self {
            pin1: gpio1.get(pin1).unwrap().into_output(),
            positive_relay,
        }
    }
}

impl Motor for WorkMotor {
    /// Rotate the work motor clockwise (CW).
    fn cw(&mut self) {
        if self.positive_relay {
            self.pin1.set_high();
        } else {
            self.pin1.set_low();
        }
    }

    /// Rotate the work motor counterclockwise (CCW).
    /// Note: CCW is not supported, as it is handled by a relay, not a motor driver.
    fn ccw(&mut self) {
        panic!("Work Motor CCW Not Implemented.")
    }

    /// Stop the work motor.
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
        // Left motor test
        let mut dml = DriveMotor::new(22, 23, 1.0);

        // Left CW
        println!("Left motor CW test power: {}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.power = 0.7;
        println!("Left motor CW test power: {}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.power = 0.5;
        println!("Left motor CW test power: {}", dml.power);
        dml.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.stop();

        // Left CCW
        dml.power = 1.0;
        println!("Left motor CCW test power: {}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.power = 0.7;
        println!("Left motor CCW test power: {}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.power = 0.5;
        println!("Left motor CCW test power: {}", dml.power);
        dml.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dml.stop();

        // Right motor test
        let mut dmr = DriveMotor::new(24, 25, 1.0);

        // Right CW
        println!("Right motor CW test power: {}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.power = 0.7;
        println!("Right motor CW test power: {}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.power = 0.5;
        println!("Right motor CW test power: {}", dmr.power);
        dmr.cw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.stop();

        // Right CCW
        dmr.power = 1.0;
        println!("Right motor CCW test power: {}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.power = 0.7;
        println!("Right motor CCW test power: {}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.power = 0.5;
        println!("Right motor CCW test power: {}", dmr.power);
        dmr.ccw();
        thread::sleep(time::Duration::from_millis(2000));

        dmr.stop();
    }

    #[test]
    fn work_motor_test() {
        let mut wm = WorkMotor::new(14, false);
        println!("Work motor CW test");
        wm.cw();
        thread::sleep(time::Duration::from_millis(2000));
        println!("Work motor stop test");
        wm.stop();
        thread::sleep(time::Duration::from_millis(5000));
    }
}
