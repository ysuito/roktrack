//! Provides miscellaneous devices.

use rppal::gpio::Gpio;

/// Defines the LimitSwitch trait.
///
/// A LimitSwitch is a simple device that reports whether a limit has been reached or not.
pub trait LimitSwitch {
    /// Get the state of the LimitSwitch.
    ///
    /// Returns `true` if the limit is reached, `false` otherwise.
    fn get(&self) -> bool;
}

/// Represents a Bumper used to detect obstacles.
pub struct Bumper {
    pub switch: rppal::gpio::InputPin,
}

impl Bumper {
    /// Creates a new Bumper instance.
    ///
    /// # Arguments
    ///
    /// * `pin` - GPIO pin number for the bumper.
    ///
    pub fn new(pin: u8) -> Self {
        let gpio = Gpio::new().unwrap();

        Self {
            switch: gpio.get(pin).unwrap().into_input_pullup(),
        }
    }
}

impl LimitSwitch for Bumper {
    /// Get the state of the Bumper.
    ///
    /// Returns `true` if the bumper is pressed, indicating an obstacle, `false` otherwise.
    fn get(&self) -> bool {
        self.switch.is_high()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bumper_initial_state_test() {
        let bumper = Bumper::new(24);
        assert!(!bumper.get());
    }
}
