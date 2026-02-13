//! Live demonstration of delegation primitives

use nexcore_vigilance::primitives::delegation::{
    DelegationConfidence, DelegationRouter, ErrorCost, ReviewProtocol, ReviewResult,
    TaskCharacteristics,
};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("       DELEGATION ROUTER - LIVE TASK ROUTING DEMO");
    println!("═══════════════════════════════════════════════════════════\n");

    // Real tasks from this session
    let tasks = vec![
        (
            "Generate tests for 112 MCP tools",
            TaskCharacteristics {
                item_count: 112,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: ErrorCost::Low,
            },
        ),
        (
            "Design delegation architecture",
            TaskCharacteristics {
                item_count: 5,
                is_repetitive: false,
                has_structure: true,
                needs_reasoning: true,
                is_novel: true,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: ErrorCost::High,
            },
        ),
        (
            "Review PR with security implications",
            TaskCharacteristics {
                item_count: 1,
                is_repetitive: false,
                has_structure: false,
                needs_reasoning: true,
                is_novel: false,
                is_sensitive: true,
                is_multimodal: false,
                error_cost: ErrorCost::Critical,
            },
        ),
        (
            "Process 50 FAERS case reports",
            TaskCharacteristics {
                item_count: 50,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: ErrorCost::Medium,
            },
        ),
        (
            "Analyze medical images for signals",
            TaskCharacteristics {
                item_count: 10,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: true,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: true,
                error_cost: ErrorCost::High,
            },
        ),
    ];

    println!(
        "┌─────────────────────────────────────┬───────────────┬────────┬─────────────────────────────┐"
    );
    println!(
        "│ Task                                │ Model         │ Conf.  │ Rationale                   │"
    );
    println!(
        "├─────────────────────────────────────┼───────────────┼────────┼─────────────────────────────┤"
    );

    for (name, task) in &tasks {
        let decision = DelegationRouter::route(task);
        let model_name = format!("{:?}", decision.model);
        println!(
            "│ {:<35} │ {:<13} │ {:>5.1}% │ {:<27} │",
            &name[..name.len().min(35)],
            model_name,
            decision.confidence * 100.0,
            decision.rationale
        );
    }
    println!(
        "└─────────────────────────────────────┴───────────────┴────────┴─────────────────────────────┘"
    );

    // Demonstrate review protocol
    println!("\n═══════════════════════════════════════════════════════════");
    println!("       REVIEW PROTOCOL - 5-STAGE VALIDATION");
    println!("═══════════════════════════════════════════════════════════\n");

    let mut protocol = ReviewProtocol::new(2); // max 2 retries

    let stages: Vec<(&str, bool, Vec<String>)> = vec![
        ("Generated", true, vec![]),
        ("Compiled", true, vec![]),
        ("Linted", false, vec!["unused variable".to_string()]),
        ("Linted (retry)", true, vec![]),
        ("SpotChecked", true, vec![]),
    ];

    for (stage, passed, issues) in stages {
        let result = ReviewResult {
            phase: protocol.current,
            passed,
            issues: issues.clone(),
        };

        let status = if passed { "✓" } else { "✗" };
        let issue_str = if issues.is_empty() {
            String::new()
        } else {
            format!(" ({})", issues.join(", "))
        };

        println!("  {:?} {} {}{}", protocol.current, status, stage, issue_str);

        protocol.advance(result);
    }

    println!(
        "\n  Final: {} (retries: {})",
        if protocol.accepted() {
            "ACCEPTED ✓"
        } else {
            "PENDING"
        },
        protocol.retries
    );

    // Demonstrate confidence scoring
    println!("\n═══════════════════════════════════════════════════════════");
    println!("       CONFIDENCE SCORING - MULTI-DIMENSIONAL");
    println!("═══════════════════════════════════════════════════════════\n");

    let scenarios = [
        ("High patterns, high volume, tolerant", 3, 100, 0.8),
        ("Low patterns, low volume, strict", 1, 5, 0.2),
        ("Medium all", 2, 50, 0.5),
    ];

    for (name, patterns, count, tolerance) in scenarios {
        let conf = DelegationConfidence::new(patterns, count, tolerance);
        let score = conf.compute();

        println!("  {} → {:.1}%", name, score.total * 100.0);
        for dim in &score.dimensions {
            println!(
                "    └─ {}: {:.2} × {:.1} = {:.2}",
                dim.name,
                dim.value,
                dim.weight,
                dim.contribution()
            );
        }
        println!();
    }

    route_session_task();
}

// --- LIVE SESSION ROUTING ---
// Route the actual FRIDAY→Vigil rename task from earlier

fn route_session_task() {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("       LIVE SESSION TASK ROUTING");
    println!("═══════════════════════════════════════════════════════════\n");

    let rename_task = TaskCharacteristics {
        item_count: 22,      // 22 occurrences to rename
        is_repetitive: true, // Same operation repeated
        has_structure: true, // Code files with patterns
        needs_reasoning: false,
        is_novel: false,
        is_sensitive: false,
        is_multimodal: false,
        error_cost: ErrorCost::Medium,
    };

    let decision = DelegationRouter::route(&rename_task);

    println!("  Task: Rename FRIDAY → Vigil (22 occurrences, 9 files)");
    println!("  ────────────────────────────────────────────────────");
    println!("  Routed to: {:?}", decision.model);
    println!("  Confidence: {:.1}%", decision.confidence * 100.0);
    println!("  Rationale: {}", decision.rationale);
    println!();

    // What about the current task (show delegation demo)?
    let demo_task = TaskCharacteristics {
        item_count: 1,
        is_repetitive: false,
        has_structure: true,
        needs_reasoning: true,
        is_novel: false,
        is_sensitive: false,
        is_multimodal: false,
        error_cost: ErrorCost::Low,
    };

    let decision2 = DelegationRouter::route(&demo_task);
    println!("  Task: Demonstrate delegation system (this task!)");
    println!("  ────────────────────────────────────────────────────");
    println!("  Routed to: {:?}", decision2.model);
    println!("  Confidence: {:.1}%", decision2.confidence * 100.0);
    println!("  Rationale: {}", decision2.rationale);
}
