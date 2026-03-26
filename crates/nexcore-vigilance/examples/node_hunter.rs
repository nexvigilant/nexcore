//! Node Hunter Simulation
//!
//! Simulation of structural signal detection and isolation within a VDAG pipeline.
//!
//! This script:
//! 1. Initializes a VDAG pipeline with 5 nodes.
//! 2. Attaches a NodeSignalScanner.
//! 3. Simulates node behaviors (signals).
//! 4. Identifies the "Anomalous Node" using PRR logic.
//! 5. Executes Isolation (∂).

use nexcore_vigilance::network_nodes::{NetworkNode, NodeSignal, NodeSignalScanner};
use nexcore_vigilance::vdag::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Node Hunter Simulation...");

    // 1. Setup VDAG Nodes (Conceptual Entities)
    let node_ids = vec!["VALIDATOR", "PARSER", "EXECUTOR", "TELEMETRY", "BRIDGE"];
    let mut scanner = NodeSignalScanner::new();
    scanner.total_reports = 5000; // Simulated network background volume

    // 2. Initialize Network Nodes with behavioral mappings
    let mut network_nodes: HashMap<String, NetworkNode> = HashMap::new();
    for id in &node_ids {
        network_nodes.insert(id.to_string(), NetworkNode::new(id.to_string()));
    }

    // 3. Simulate Signals
    println!("📡 Monitoring network signals...");

    // Most nodes behave normally
    for id in &node_ids {
        let node = network_nodes
            .get_mut(*id)
            .ok_or_else(|| format!("node '{id}' not found in network"))?;
        // Normal baseline activity
        node.signals.push(NodeSignal {
            pattern_id: "HEARTBEAT".into(),
            intensity: 1.0,
            timestamp: 1.0,
        });
        node.signals.push(NodeSignal {
            pattern_id: "SUCCESS".into(),
            intensity: 0.9,
            timestamp: 1.1,
        });
    }

    // "EXECUTOR" node starts exhibiting "LATENCY_SPIKE" (The Target Pattern)
    println!("⚠️  Anomalous behavior detected in 'EXECUTOR' node...");
    let executor = network_nodes
        .get_mut("EXECUTOR")
        .ok_or("node 'EXECUTOR' not found in network")?;
    for i in 0..10 {
        executor.signals.push(NodeSignal {
            pattern_id: "LATENCY_SPIKE".into(),
            intensity: 0.85,
            timestamp: 2.0 + (i as f64 * 0.1),
        });
    }

    // Background has a few other spikes (noise)
    let parser = network_nodes
        .get_mut("PARSER")
        .ok_or("node 'PARSER' not found in network")?;
    parser.signals.push(NodeSignal {
        pattern_id: "LATENCY_SPIKE".into(),
        intensity: 0.4,
        timestamp: 2.5,
    });

    // 4. Run Structural Signal Detection (Node Hunting)
    for node in network_nodes.values() {
        scanner.add_node(node.clone());
    }

    println!("🔍 Running Disproportionality Scan for 'LATENCY_SPIKE'...");
    let matches = scanner.find_nodes("LATENCY_SPIKE");

    if matches.is_empty() {
        println!("✅ No nodes exceeded disproportionality thresholds.");
    } else {
        for (id, iden) in matches {
            println!(
                "🚨 SIGNAL MATCH: Node '{}' identified as source of instability!",
                id
            );
            println!("   - PRR: {:.2} (Threshold: 2.0)", iden.prr);
            println!("   - Confidence: {:.2}", iden.confidence);

            // 5. Isolation (∂ Boundary)
            println!(
                "🛡️  Establishing Boundary Primitive (∂) for node '{}'...",
                id
            );
            let target_node = network_nodes
                .get_mut(&id)
                .ok_or_else(|| format!("node '{id}' not found in network"))?;
            target_node.isolate();
            println!(
                "🚫 Node '{}' isolated from network. is_isolated: {}",
                id, target_node.is_isolated
            );
        }
    }

    println!("🏁 Simulation complete.");
    Ok(())
}
