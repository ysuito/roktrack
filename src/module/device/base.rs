//! Provide miscellaneous devices
//!

use rppal::gpio::Gpio;

/// Simple LimitSwitch
///
pub trait LimitSwitch {
    fn get(&self) -> bool;
}

/// Bumper to detect obstacles
///
pub struct Bumper {
    pub switch: rppal::gpio::InputPin,
}

/// Bumper's methods
///
impl Bumper {
    /// Bumpers's constructor
    ///
    pub fn new(pin: u8) -> Self {
        let gpio = Gpio::new().unwrap();
        Self {
            switch: gpio.get(pin).unwrap().into_input_pullup(),
        }
    }
}
/// Implement LimitSwitch for Bumper.
///
impl LimitSwitch for Bumper {
    fn get(&self) -> bool {
        self.switch.is_high()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bumper_intitial_state_test() {
        let bumper = Bumper::new(24);
        assert!(!bumper.get());
    }
}
