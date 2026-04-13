//! # E-Stop Cascade (Hard Real-Time)
pub struct EStopController<W: HardwareWatchdog> {
    pub watchdog: W,
}
impl<W: HardwareWatchdog> EStopController<W> {
    pub fn poll(&mut self) {
        self.watchdog.kick();
    }
}
