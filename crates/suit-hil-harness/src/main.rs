use suit_flight::ControlBridge;
use suit_perception::perception_engine::PerceptionEngine;
use suit_power_core::engine::PowerEngine;
use wksp_types::perception::InertialMessage;

fn main() {
    println!("--- NexSuit HIL Harness: System Start ---");
    let mut perception = PerceptionEngine::new("model/intent.bin".to_string());
    let mut power = PowerEngine::new();

    // Simulate one system tick
    println!("Simulating 10ms System Tick...");
    let world = perception.update(
        &suit_perception::vestibular::InertialState::default(),
        &suit_perception::vestibular::MagnetometerData { field: [0.0; 3] },
        &suit_perception::vestibular::BarometricData {
            pressure: 1013.25,
            temperature: 20.0,
        },
        &suit_perception::proprioceptive::BodyState {
            joint_angles: vec![],
            joint_velocities: vec![],
            heart_rate: 60,
            spo2: 98,
            foot_pressure: vec![],
        },
        &None,
    );

    let (soc, shed) = power.update(
        400.0,
        10.0,
        25.0,
        8000.0,
        &suit_power_core::mission::MissionForecast {
            load_forecast: vec![],
        },
    );

    let cmd =
        ControlBridge::translate_perception(&world, suit_perception::intent::Intent::Standing);

    println!(
        "Harness Output: WorldState: {:?}, PowerSOC: {:?}, FlightTarget: {:?}",
        world, soc.soc, cmd.target_vector
    );
    println!("--- NexSuit HIL Harness: Tick Complete ---");
}
