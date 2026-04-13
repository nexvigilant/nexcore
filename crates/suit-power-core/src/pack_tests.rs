#[cfg(test)]
mod tests {
    use crate::pack::{BatteryModule, MergeState};

    #[test]
    fn test_hot_swap_handshake() {
        let mut module = BatteryModule::new(1);
        module.voltage = 400.0;
        let bus_voltage = 398.0;

        // Disconnected -> Precharge
        module.tick(bus_voltage);
        assert_eq!(module.state, MergeState::Precharge);

        // Precharge -> Ramping
        module.tick(bus_voltage);
        assert_eq!(module.state, MergeState::Ramping);

        // Ramping -> Merged
        module.tick(bus_voltage);
        assert_eq!(module.state, MergeState::Merged);
    }
}
