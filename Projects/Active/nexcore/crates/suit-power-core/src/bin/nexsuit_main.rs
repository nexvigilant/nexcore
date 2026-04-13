use std::thread;
use std::time::Duration;
use suit_power_core::hook::NexSuitHook;

fn main() {
    println!("Initializing NexSuit Autonomous Executive...");
    let mut hook = NexSuitHook::new();
    println!("NexSuit system initialized. Executing control loop...");

    loop {
        hook.tick();
        thread::sleep(Duration::from_millis(100)); // 10Hz tick
    }
}
