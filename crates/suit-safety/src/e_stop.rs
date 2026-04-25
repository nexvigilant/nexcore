//! # E-Stop Cascade (Hard Real-Time)
use crate::hardware_watchdog::HardwareWatchdog;

pub struct EStopController<W: HardwareWatchdog> {
    pub watchdog: W,
}
impl<W: HardwareWatchdog> EStopController<W> {
    pub fn poll(&mut self) {
        self.watchdog.kick();
    }
}
