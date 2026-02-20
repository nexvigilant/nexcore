//! Generate sample SVG files for all visualization types.
//!
//! Run with: cargo run --example generate_samples -p nexcore-viz

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "/tmp/viz-samples".to_string());
    let _ = std::fs::create_dir_all(&out_dir);

    // 1. STEM Taxonomy Sunburst
    let taxonomy = nexcore_viz::taxonomy::standard_taxonomy();
    let svg = nexcore_viz::render_taxonomy(&taxonomy, "STEM Taxonomy — 32 Traits");
    let path = format!("{out_dir}/taxonomy.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 2. Science Loop
    let steps = nexcore_viz::science_loop::science_loop();
    let svg = nexcore_viz::render_science_loop(&steps, "SCIENCE");
    let path = format!("{out_dir}/science_loop.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 3. Chemistry Loop
    let steps = nexcore_viz::science_loop::chemistry_loop();
    let svg = nexcore_viz::render_science_loop(&steps, "CHEMISTRY");
    let path = format!("{out_dir}/chemistry_loop.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 4. Type Composition: Machine<I,O> (T3)
    let comp = nexcore_viz::TypeComposition {
        type_name: "Machine<I,O>".into(),
        tier: "T3".into(),
        primitives: vec![
            nexcore_viz::PrimitiveNode {
                name: "Mapping".into(),
                symbol: "\u{03bc}".into(),
                role: "I -> O transformation".into(),
            },
            nexcore_viz::PrimitiveNode {
                name: "Sequence".into(),
                symbol: "\u{03c3}".into(),
                role: "ordered step chain".into(),
            },
            nexcore_viz::PrimitiveNode {
                name: "State".into(),
                symbol: "\u{03c2}".into(),
                role: "internal counter".into(),
            },
            nexcore_viz::PrimitiveNode {
                name: "Causality".into(),
                symbol: "\u{2192}".into(),
                role: "mechanism causal chain".into(),
            },
            nexcore_viz::PrimitiveNode {
                name: "Comparison".into(),
                symbol: "\u{03ba}".into(),
                role: "determinism".into(),
            },
            nexcore_viz::PrimitiveNode {
                name: "Quantity".into(),
                symbol: "N".into(),
                role: "confidence, counts".into(),
            },
        ],
        dominant: Some("Mapping".into()),
        confidence: 0.80,
    };
    let svg = nexcore_viz::render_composition(&comp);
    let path = format!("{out_dir}/composition_machine.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 5. Confidence Propagation
    let claims = vec![
        nexcore_viz::Claim {
            text: "D1: Use Leptos 0.7".into(),
            confidence: 0.95,
            proof_type: "analytical".into(),
            parent: None,
        },
        nexcore_viz::Claim {
            text: "D2: Firebase REST API".into(),
            confidence: 0.90,
            proof_type: "empirical".into(),
            parent: None,
        },
        nexcore_viz::Claim {
            text: "A1: schema.rs migration".into(),
            confidence: 0.99,
            proof_type: "computational".into(),
            parent: None,
        },
        nexcore_viz::Claim {
            text: "D3: Rate limiter design".into(),
            confidence: 0.85,
            proof_type: "derived".into(),
            parent: Some(0),
        },
        nexcore_viz::Claim {
            text: "D4: Scarring threshold".into(),
            confidence: 0.80,
            proof_type: "empirical".into(),
            parent: Some(2),
        },
    ];
    let svg = nexcore_viz::render_confidence_chain(&claims, "Confidence Propagation");
    let path = format!("{out_dir}/confidence.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 6. Bounds (in-bounds)
    let bounded = nexcore_viz::BoundedValue {
        value: 0.85,
        lower: Some(0.0),
        upper: Some(1.0),
        label: "Confidence [0, 1]".into(),
    };
    let svg = nexcore_viz::render_bounds(&bounded);
    let path = format!("{out_dir}/bounds_in.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 7. Bounds (out-of-bounds)
    let bounded = nexcore_viz::BoundedValue {
        value: 15.0,
        lower: Some(0.0),
        upper: Some(10.0),
        label: "Error Rate [0, 10]".into(),
    };
    let svg = nexcore_viz::render_bounds(&bounded);
    let path = format!("{out_dir}/bounds_out.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    // 8. DAG
    let nodes = vec![
        nexcore_viz::DagNode {
            id: "N".into(),
            label: "Quantity (N)".into(),
            color: None,
        },
        nexcore_viz::DagNode {
            id: "->".into(),
            label: "Causality (->)".into(),
            color: None,
        },
        nexcore_viz::DagNode {
            id: "sigma".into(),
            label: "Sequence (sigma)".into(),
            color: Some("#34d399".into()),
        },
        nexcore_viz::DagNode {
            id: "mu".into(),
            label: "Mapping (mu)".into(),
            color: Some("#60a5fa".into()),
        },
        nexcore_viz::DagNode {
            id: "kappa".into(),
            label: "Comparison (kappa)".into(),
            color: Some("#fbbf24".into()),
        },
        nexcore_viz::DagNode {
            id: "rho".into(),
            label: "Recursion (rho)".into(),
            color: Some("#f472b6".into()),
        },
    ];
    let edges = vec![
        nexcore_viz::DagEdge {
            from: "N".into(),
            to: "sigma".into(),
        },
        nexcore_viz::DagEdge {
            from: "N".into(),
            to: "mu".into(),
        },
        nexcore_viz::DagEdge {
            from: "->".into(),
            to: "sigma".into(),
        },
        nexcore_viz::DagEdge {
            from: "->".into(),
            to: "kappa".into(),
        },
        nexcore_viz::DagEdge {
            from: "sigma".into(),
            to: "rho".into(),
        },
        nexcore_viz::DagEdge {
            from: "mu".into(),
            to: "rho".into(),
        },
    ];
    let svg = nexcore_viz::render_dag(&nodes, &edges, "Lex Primitiva DAG (partial)");
    let path = format!("{out_dir}/dag.svg");
    std::fs::write(&path, &svg).ok();
    println!("Wrote {path} ({} bytes)", svg.len());

    println!("\nAll 8 samples generated in {out_dir}/");
}
