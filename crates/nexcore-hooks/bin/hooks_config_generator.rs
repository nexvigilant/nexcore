//! Hook Config Generator
//!
//! Generates Claude Code settings.json from the hooks catalog.
//!
//! Usage:
//!   hooks-config-generator --tier dev     # Generate dev config
//!   hooks-config-generator --tier review  # Generate review config
//!   hooks-config-generator --tier deploy  # Generate deploy config
//!   hooks-config-generator --list         # List all hooks
//!   hooks-config-generator --list --tier dev  # List hooks in tier

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let catalog_path = get_catalog_path();
    let catalog = match fs::read_to_string(&catalog_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading catalog: {}", e);
            std::process::exit(1);
        }
    };

    if args.contains(&"--list".to_string()) {
        let tier = args
            .iter()
            .position(|a| a == "--tier")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str());
        list_hooks(&catalog, tier);
    } else if let Some(pos) = args.iter().position(|a| a == "--tier") {
        if let Some(tier) = args.get(pos + 1) {
            generate_config(&catalog, tier);
        } else {
            eprintln!("Error: --tier requires a value (dev, review, deploy)");
            std::process::exit(1);
        }
    } else {
        print_usage();
    }
}

fn get_catalog_path() -> PathBuf {
    // Try multiple locations
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let paths = [
        format!("{}/nexcore/crates/nexcore-hooks/hooks-catalog.yaml", home),
        "hooks-catalog.yaml".to_string(),
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return PathBuf::from(path);
        }
    }

    PathBuf::from(&paths[0])
}

fn print_usage() {
    eprintln!("Hook Config Generator");
    eprintln!("");
    eprintln!("Usage:");
    eprintln!("  hooks-config-generator --tier <dev|review|deploy>");
    eprintln!("  hooks-config-generator --list [--tier <tier>]");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  hooks-config-generator --tier dev > ~/.claude/settings.json");
    eprintln!("  hooks-config-generator --list --tier review");
}

fn list_hooks(catalog: &str, filter_tier: Option<&str>) {
    let mut current_event = String::new();
    let mut hooks_by_event: HashMap<String, Vec<(String, String, Vec<String>)>> = HashMap::new();

    for line in catalog.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Event header (e.g., "SessionStart:" or "PreToolUse:Bash:")
        if !trimmed.starts_with('-') && !trimmed.starts_with(' ') && trimmed.ends_with(':') {
            current_event = trimmed.trim_end_matches(':').to_string();
            continue;
        }

        // Hook name
        if trimmed.ends_with(':') && !trimmed.contains(' ') {
            let hook_name = trimmed.trim_end_matches(':').to_string();

            // Read subsequent lines for description and tier
            let mut description = String::new();
            let mut tiers = Vec::new();

            // Parse the hook block (simplified - just look for description and tier in catalog)
            for check_line in catalog.lines() {
                if check_line.contains(&format!("{}:", hook_name)) {
                    // Found the hook, now look for its properties
                    let mut in_hook = false;
                    for prop_line in catalog.lines() {
                        if prop_line.trim() == format!("{}:", hook_name) {
                            in_hook = true;
                            continue;
                        }
                        if in_hook {
                            if prop_line.trim().starts_with("description:") {
                                description = prop_line
                                    .split(':')
                                    .skip(1)
                                    .collect::<Vec<_>>()
                                    .join(":")
                                    .trim()
                                    .trim_matches('"')
                                    .to_string();
                            }
                            if prop_line.trim().starts_with("tier:") {
                                let tier_str = prop_line.split(':').nth(1).unwrap_or("").trim();
                                // Parse [dev, review, deploy] format
                                tiers = tier_str
                                    .trim_matches(|c| c == '[' || c == ']')
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                            // Stop at next hook or event
                            if !prop_line.starts_with(' ') && !prop_line.is_empty() {
                                break;
                            }
                        }
                    }
                    break;
                }
            }

            // Filter by tier if specified
            if let Some(ft) = filter_tier {
                if !tiers.contains(&ft.to_string()) {
                    continue;
                }
            }

            hooks_by_event
                .entry(current_event.clone())
                .or_default()
                .push((hook_name, description, tiers));
        }
    }

    // Print results
    let mut events: Vec<_> = hooks_by_event.keys().collect();
    events.sort();

    let mut total = 0;
    for event in events {
        if let Some(hooks) = hooks_by_event.get(event) {
            if hooks.is_empty() {
                continue;
            }
            println!("\n{}", event);
            println!("{}", "=".repeat(event.len()));
            for (name, desc, tiers) in hooks {
                let tier_str = if tiers.is_empty() {
                    "none".to_string()
                } else {
                    tiers.join(", ")
                };
                println!("  {} [{}]", name, tier_str);
                if !desc.is_empty() {
                    println!("    {}", desc);
                }
                total += 1;
            }
        }
    }

    println!("\nTotal: {} hooks", total);
    if let Some(t) = filter_tier {
        println!("(filtered by tier: {})", t);
    }
}

fn generate_config(catalog: &str, tier: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let hooks_bin = format!("{}/nexcore/crates/nexcore-hooks/target/release", home);

    // Parse catalog and collect hooks for this tier
    let mut hooks_by_event: HashMap<String, Vec<(String, u32, String)>> = HashMap::new();
    let mut current_event = String::new();
    let mut current_hook = String::new();
    let mut current_timeout = 3u32;
    let mut current_tiers: Vec<String> = Vec::new();
    let mut current_matcher = String::new();

    for line in catalog.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Event header
        if !trimmed.starts_with(' ') && trimmed.ends_with(':') && !trimmed.contains("description") {
            // Save previous hook if valid
            if !current_hook.is_empty() && current_tiers.contains(&tier.to_string()) {
                let event_key = if current_matcher.is_empty() {
                    current_event.clone()
                } else {
                    format!(
                        "{}:{}",
                        current_event.split(':').next().unwrap_or(&current_event),
                        current_matcher
                    )
                };
                hooks_by_event.entry(event_key).or_default().push((
                    current_hook.clone(),
                    current_timeout,
                    current_matcher.clone(),
                ));
            }

            current_event = trimmed.trim_end_matches(':').to_string();
            current_hook.clear();
            current_tiers.clear();
            current_matcher.clear();
            current_timeout = 3;
            continue;
        }

        // Hook name (indented, ends with :)
        if line.starts_with("  ")
            && !line.starts_with("    ")
            && trimmed.ends_with(':')
            && !trimmed.contains(' ')
        {
            // Save previous hook if valid
            if !current_hook.is_empty() && current_tiers.contains(&tier.to_string()) {
                let event_key = if current_matcher.is_empty() {
                    current_event.clone()
                } else {
                    format!(
                        "{}:{}",
                        current_event.split(':').next().unwrap_or(&current_event),
                        current_matcher
                    )
                };
                hooks_by_event.entry(event_key).or_default().push((
                    current_hook.clone(),
                    current_timeout,
                    current_matcher.clone(),
                ));
            }

            current_hook = trimmed.trim_end_matches(':').to_string();
            current_tiers.clear();
            current_matcher.clear();
            current_timeout = 3;
            continue;
        }

        // Hook properties
        if line.starts_with("    ") && !current_hook.is_empty() {
            if trimmed.starts_with("tier:") {
                let tier_str = trimmed.split(':').nth(1).unwrap_or("").trim();
                current_tiers = tier_str
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            if trimmed.starts_with("timeout:") {
                current_timeout = trimmed
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(3);
            }
            if trimmed.starts_with("matcher:") {
                current_matcher = trimmed
                    .split(':')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    // Save last hook
    if !current_hook.is_empty() && current_tiers.contains(&tier.to_string()) {
        let event_key = if current_matcher.is_empty() {
            current_event.clone()
        } else {
            format!(
                "{}:{}",
                current_event.split(':').next().unwrap_or(&current_event),
                current_matcher
            )
        };
        hooks_by_event.entry(event_key).or_default().push((
            current_hook.clone(),
            current_timeout,
            current_matcher.clone(),
        ));
    }

    // Generate JSON
    let tier_upper = tier.to_uppercase();
    let mode_msg = format!("{} MODE: Generated from hooks-catalog.yaml", tier_upper);

    println!(
        r#"{{
  "alwaysThinkingEnabled": true,
  "cleanupPeriodDays": 7,
  "outputStyle": "Explanatory",
  "autoUpdatesChannel": "stable",
  "companyAnnouncements": [
    "100% Rust policy: Flag any Python for migration",
    "{}"
  ],
  "env": {{
    "RUST_BACKTRACE": "1",
    "CARGO_TERM_COLOR": "always"
  }},
  "attribution": {{
    "commit": "Authored by: Matthew Campion, PharmD; NexVigilant",
    "pr": "Authored by: Matthew Campion, PharmD; NexVigilant"
  }},
  "sandbox": {{
    "enabled": false,
    "autoAllowBashIfSandboxed": true
  }},
  "permissions": {{
    "allow": [
      "Bash(cargo *)",
      "Bash(git *)",
      "Bash(gh *)",
      "Bash(ls *)",
      "Bash(cat *)",
      "Bash(grep *)",
      "Bash(find *)",
      "Bash(mkdir *)",
      "Bash(cp *)",
      "Bash(mv *)",
      "Bash(rm *)",
      "Bash(nexcore *)",
      "mcp__nexcore__*",
      "mcp__claude-code-docs__*",
      "mcp__rust-lang__*"
    ],
    "defaultMode": "acceptEdits"
  }},"#,
        mode_msg
    );

    // Generate hooks section
    println!(r#"  "hooks": {{"#);

    let mut event_entries = Vec::new();

    // Group by base event
    let mut events_order = vec![
        "SessionStart",
        "SessionEnd",
        "UserPromptSubmit",
        "PreToolUse:Bash",
        "PreToolUse:Edit|Write",
        "PreToolUse:Task",
        "PostToolUse",
        "SubagentStart",
        "PreCompact",
        "PermissionRequest",
        "PostToolUseFailure",
        "Stop",
        "Setup",
    ];

    for event in &events_order {
        if let Some(hooks) = hooks_by_event.get(*event) {
            if hooks.is_empty() {
                continue;
            }

            let (base_event, matcher) = if event.contains(':') {
                let parts: Vec<&str> = event.splitn(2, ':').collect();
                (parts[0], parts.get(1).copied().unwrap_or(""))
            } else {
                (*event, "")
            };

            let mut hook_entries = Vec::new();
            for (hook_name, timeout, _) in hooks {
                hook_entries.push(format!(
                    r#"          {{
            "type": "command",
            "command": "{}/{}",
            "timeout": {}
          }}"#,
                    hooks_bin, hook_name, timeout
                ));
            }

            let matcher_json = if matcher.is_empty() {
                r#""matcher": "","#.to_string()
            } else {
                format!(r#""matcher": "{}","#, matcher)
            };

            event_entries.push(format!(
                r#"    "{}": [
      {{
        {}
        "hooks": [
{}
        ]
      }}
    ]"#,
                base_event,
                matcher_json,
                hook_entries.join(",\n")
            ));
        }
    }

    println!("{}", event_entries.join(",\n"));

    println!(
        r#"  }},
  "statusLine": {{
    "type": "command",
    "command": "{}/.claude/statusline.sh"
  }},
  "mcpServers": {{
    "nexcore": {{
      "command": "${{HOME}}/.cargo/bin/nexcore-mcp",
      "args": []
    }},
    "claude-code-docs": {{
      "command": "${{HOME}}/.cargo/bin/claude-code-docs-mcp",
      "args": []
    }},
    "rust-lang": {{
      "command": "uv",
      "args": ["run", "--directory", "${{HOME}}/.claude/mcp-servers/mcp-rust-lang", "python", "-m", "mcp_rust_lang.main"]
    }}
  }}
}}"#,
        home
    );
}
