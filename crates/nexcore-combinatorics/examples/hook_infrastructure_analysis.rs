//! Real-world application: Dudeney combinatorics applied to hook infrastructure
//!
//! This analysis uses the 98 hooks across 8 lifecycle events from
//! ~/.claude/settings.json to demonstrate cross-domain transfer from
//! Victorian puzzle mathematics to modern infrastructure engineering.
//!
//! Run: cargo run --example hook_infrastructure_analysis

use nexcore_combinatorics::{
    catalan, cycle_decomposition, derangement, derangement_probability, grid_paths,
    josephus::{elimination_order, josephus},
    linear_extensions::count_from_subgraphs,
};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Hook Infrastructure Analysis via Dudeney Combinatorics");
    println!("═══════════════════════════════════════════════════════════\n");

    // ── 1. Linear Extensions: Stop Hook Execution Orderings ─────────────
    //
    // The 8 Stop hooks have dependency structure:
    //   Chain 1: brain-sync → knowledge-distiller → session-reflector (3, strict order)
    //   Chain 2: session-handoff → waste-collector (2, handoff before cleanup)
    //   Chain 3: insight-generator (1, independent)
    //   Chain 4: reflex-brain-save → guardian-decay (2, telemetry before decay)
    //
    // How many valid orderings exist?

    println!("1. STOP HOOK EXECUTION ORDERINGS (Linear Extensions — Dudeney #279+#253)");
    println!("   ────────────────────────────────────────────────────────");

    let stop_result = count_from_subgraphs(&[
        ("brain-sync→distiller→reflector", 3, 1),
        ("handoff→waste-collector", 2, 1),
        ("insight-generator", 1, 1),
        ("reflex-brain-save→guardian-decay", 2, 1),
    ]);

    println!("   Subgraphs:");
    for sg in &stop_result.subgraphs {
        println!(
            "     {:40} nodes={} internal={}",
            sg.name, sg.node_count, sg.internal_orderings
        );
    }
    println!(
        "   Multinomial factor (interleaving):  {}",
        stop_result.multinomial_factor
    );
    println!(
        "   Internal orderings product:         {}",
        stop_result.internal_factor
    );
    println!(
        "   TOTAL valid execution orderings:    {}",
        stop_result.total_orderings
    );
    println!("   Total hooks: {}\n", stop_result.total_nodes);

    // ── 2. SessionStart Hook Orderings ──────────────────────────────────
    //
    // 9 SessionStart hooks with dependencies:
    //   Chain 1: guardian-session-sensor → session-restore → brain-hygiene (3, must load state first)
    //   Chain 2: learning-consumer → skill-synthesizer (2, consume before synthesize)
    //   Chain 3: handoff-loader → brain-preload-guardrails (2, load before guard)
    //   Chain 4: graph-optimizer → trend-dashboard (2, optimize before display)

    println!("2. SESSION START ORDERINGS (Linear Extensions)");
    println!("   ────────────────────────────────────────────────────────");

    let start_result = count_from_subgraphs(&[
        ("guardian→restore→hygiene", 3, 1),
        ("learning→synthesizer", 2, 1),
        ("handoff→guardrails", 2, 1),
        ("optimizer→dashboard", 2, 1),
    ]);

    println!("   TOTAL valid orderings: {}", start_result.total_orderings);
    println!("   (of 9! = 362880 unrestricted permutations)\n");

    // ── 3. Cycle Decomposition: PreToolUse Hook Reordering ──────────────
    //
    // The 9 PreToolUse matchers are currently in order:
    //   [Edit, Edit|Write, Write, Bash, Skill, WebFetch, Read, AllTools, nexcore]
    //
    // If a misconfiguration scrambled them to:
    //   [Bash, Read, Edit, AllTools, Write, Skill, nexcore, Edit|Write, WebFetch]
    // (mapping: 0→3, 1→6, 2→0, 3→7, 4→2, 5→5, 6→8, 7→1, 8→4 — this perm is
    //  [3,6,0,7,2,5,8,1,4])
    //
    // How many swaps to restore?

    println!("3. PRETOOLUSE MATCHER REORDERING (Cycle Decomposition — Dudeney #238)");
    println!("   ────────────────────────────────────────────────────────");

    let scrambled = [3, 6, 0, 7, 2, 5, 8, 1, 4];
    let matchers = [
        "Edit",
        "Edit|Write",
        "Write",
        "Bash",
        "Skill",
        "WebFetch",
        "Read",
        "AllTools",
        "nexcore",
    ];

    let result = cycle_decomposition(&scrambled);
    println!("   Matchers: {:?}", matchers);
    println!("   Scrambled permutation: {:?}", scrambled);
    println!("   Cycles found: {}", result.num_cycles);
    for cycle in &result.cycles {
        let names: Vec<&str> = cycle.elements.iter().map(|&i| matchers[i]).collect();
        println!("     ({}) — length {}", names.join(" → "), cycle.length);
    }
    println!("   Fixed points: {} (already correct)", result.fixed_points);
    println!(
        "   MINIMUM SWAPS TO RESTORE: {} (Dudeney: n-cycles = {}-{} = {})\n",
        result.min_transpositions, result.n, result.num_cycles, result.min_transpositions
    );

    // ── 4. Josephus: Hook Timeout Budget Elimination ────────────────────
    //
    // 11 UserPromptSubmit hooks compete for a shared timeout budget.
    // If we must eliminate every 3rd hook (by timeout consumption),
    // which hook survives?

    println!("4. PROMPT HOOK SURVIVAL (Josephus Problem — Dudeney #232)");
    println!("   ────────────────────────────────────────────────────────");

    let prompt_hooks = [
        "guardian-prompt-sensor",
        "brain-think",
        "skill-injector",
        "vocabulary-counter",
        "lesson-recall",
        "skill-reflex",
        "prompt-algorithm-miner",
        "route-advisor",
        "introspection-digest",
        "reflex-chain-injector",
        "guardian-kpi-checker",
    ];

    let survivor_pos = josephus(11, 3);
    let elim_order = elimination_order(11, 3);

    println!("   11 hooks, eliminating every 3rd:");
    for (step, &pos) in elim_order.iter().enumerate() {
        let pos_usize = pos as usize;
        let marker = if step == 10 { " ← SURVIVOR" } else { "" };
        println!(
            "     Step {:2}: eliminate {} (pos {}){}",
            step + 1,
            prompt_hooks[pos_usize],
            pos,
            marker
        );
    }
    println!(
        "   Survivor: {} at position {}\n",
        prompt_hooks[survivor_pos as usize], survivor_pos
    );

    // ── 5. Derangements: Configuration Chaos Analysis ───────────────────
    //
    // If all 8 Stop hooks were randomly shuffled, what's the probability
    // that NONE ends up in its correct position?

    println!("5. CONFIGURATION CHAOS (Derangements — Dudeney #267)");
    println!("   ────────────────────────────────────────────────────────");

    for n in [8u32, 9, 11, 26, 33] {
        let event_name = match n {
            8 => "Stop hooks",
            9 => "SessionStart hooks",
            11 => "UserPromptSubmit hooks",
            26 => "PostToolUse hooks",
            33 => "PreToolUse hooks",
            _ => "hooks",
        };
        let d = derangement(n);
        let p = derangement_probability(n);
        println!(
            "   D({:2}) = {:>15} — {:<24} (P={:.5}, ≈1/e)",
            n, d, event_name, p
        );
    }
    println!("   Insight: ~36.8% of random shuffles displace EVERY hook.\n");

    // ── 6. Grid Paths: Event Lattice Navigation ─────────────────────────
    //
    // A tool call traverses a lattice: 8 event types (down) × varying
    // matcher counts (right). How many distinct paths exist?

    println!("6. EVENT LATTICE PATHS (Grid Paths — Dudeney #253)");
    println!("   ────────────────────────────────────────────────────────");

    // Matchers per event type
    let event_matchers = [
        ("PreToolUse", 9u32),
        ("PostToolUse", 7),
        ("PostToolUseFailure", 1),
        ("UserPromptSubmit", 1),
        ("SessionStart", 1),
        ("Stop", 1),
        ("PreCompact", 1),
        ("SessionEnd", 1),
    ];

    for &(event, matchers_count) in &event_matchers {
        let paths = grid_paths(matchers_count, 3); // 3 steps through hook processing
        println!(
            "   {:<22} {:>2} matchers × 3 steps = {:>6} lattice paths",
            event, matchers_count, paths
        );
    }
    println!();

    // ── 7. Catalan: Hook Dependency Tree Shapes ─────────────────────────
    //
    // For n hooks with tree-structured dependencies, C(n) gives
    // the number of valid binary tree shapes.

    println!("7. HOOK DEPENDENCY TREE SHAPES (Catalan Numbers — Dudeney #279)");
    println!("   ────────────────────────────────────────────────────────");

    for n in [3u32, 5, 8, 9, 11, 14] {
        let desc = match n {
            3 => "sync PostToolUse chain",
            5 => "Edit|Write async hooks",
            8 => "Stop hooks",
            9 => "SessionStart hooks",
            11 => "UserPromptSubmit hooks",
            14 => "full lifecycle events + matcher groups",
            _ => "",
        };
        println!(
            "   C({:2}) = {:>10} — valid binary tree shapes for {} (n={})",
            n,
            catalan(n),
            desc,
            n
        );
    }
    println!();

    // ── 8. Combined Analysis: Full Infrastructure ───────────────────────

    println!("8. FULL INFRASTRUCTURE SUMMARY");
    println!("   ────────────────────────────────────────────────────────");
    println!("   Total hook entries:          98");
    println!("   Lifecycle events:            8");
    println!("   PreToolUse matchers:         9 (33 hooks)");
    println!("   PostToolUse matchers:        7 (26 hooks)");
    println!("   PostToolUseFailure:          1 (8 hooks)");
    println!("   UserPromptSubmit:            1 (11 hooks)");
    println!("   SessionStart:                1 (9 hooks)");
    println!("   Stop:                        1 (8 hooks)");
    println!("   PreCompact:                  1 (2 hooks)");
    println!("   SessionEnd:                  1 (1 hook)");
    println!();

    // Full system: if we model the entire hook system as 4 independent
    // event chains, how many valid global orderings exist?
    let full_result = count_from_subgraphs(&[
        ("SessionStart-chain", 9, stop_result.total_orderings), // reuse SessionStart's freedom
        ("PreTool→PostTool-cycle", 33 + 26, 1),                 // sequential within a tool call
        ("Error-handling", 8, 1),                               // PostToolUseFailure
        ("Stop-chain", 8, stop_result.total_orderings),         // 8 hooks, ordering freedom
    ]);

    println!(
        "   Global interleaving orderings: {}",
        full_result.total_orderings
    );
    println!(
        "   (Multinomial factor: {})",
        full_result.multinomial_factor
    );
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Dudeney → Infrastructure: Cross-Domain Transfer Complete");
    println!("═══════════════════════════════════════════════════════════");
}
