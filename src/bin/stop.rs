//! Emergency stop.
include!("../module/device/motor.rs");

fn main() {
    let mut left_motor = DriveMotor::new(22, 23, 0.0);
    let mut right_motor = DriveMotor::new(24, 25, 0.0);
    let mut work_motor1 = WorkMotor::new(14, false);
    let mut work_motor2 = WorkMotor::new(17, true);
    left_motor.stop();
    right_motor.stop();
    work_motor1.stop();
    work_motor2.stop();
}
