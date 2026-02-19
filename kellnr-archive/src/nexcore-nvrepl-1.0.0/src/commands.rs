//! Command parsing for NVREPL
//!
//! Tier: T3 (Command grounds to μ Mapping — input→variant dispatch)

use nexcore_vigilance::guardian::{OriginatorType, RiskContext};

/// Parsed command from user input
pub enum Command {
    // === Guardian (existing) ===
    Risk(RiskContext),
    Tick,
    Status,
    Reset,
    Originator(String),

    // === Signal Detection ===
    Signal { a: u64, b: u64, c: u64, d: u64 },
    Prr { a: u64, b: u64, c: u64, d: u64 },
    Ror { a: u64, b: u64, c: u64, d: u64 },
    Ic { a: u64, b: u64, c: u64, d: u64 },
    Ebgm { a: u64, b: u64, c: u64, d: u64 },

    // === Monitoring ===
    Health,
    Alerts { severity: Option<String> },
    Sensors,
    MonitorTick,

    // === Patient Safety ===
    Triage { seriousness: String },
    Priority { a: String, b: String },
    Escalation { seriousness: String },

    // === Energy ===
    Energy { budget: u64 },
    Decide { budget: u64, cost: u64, value: f64 },

    // === Meta ===
    Help,
    Exit,
    Unknown(String),
}

/// Shell-like tokenizer respecting quoted strings
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.trim().chars().peekable();

    while chars.peek().is_some() {
        skip_whitespace(&mut chars);
        if let Some(token) = extract_token(&mut chars) {
            tokens.push(token);
        }
    }
    tokens
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
}

fn extract_token(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    let first = chars.peek().copied()?;
    if first == '"' || first == '\'' {
        extract_quoted(chars, first)
    } else {
        extract_unquoted(chars)
    }
}

fn extract_quoted(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    quote: char,
) -> Option<String> {
    chars.next(); // consume opening quote
    let mut token = String::new();
    for c in chars.by_ref() {
        if c == quote {
            break;
        }
        token.push(c);
    }
    Some(token)
}

fn extract_unquoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    let mut token = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            break;
        }
        token.push(c);
        chars.next();
    }
    if token.is_empty() { None } else { Some(token) }
}

impl Command {
    pub fn parse(input: &str) -> Self {
        let parts = tokenize(input);
        if parts.is_empty() {
            return Self::Unknown(String::new());
        }
        let args: Vec<&str> = parts.iter().skip(1).map(String::as_str).collect();
        Self::from_keyword(&parts[0], &args)
    }

    fn from_keyword(keyword: &str, args: &[&str]) -> Self {
        match keyword.to_lowercase().as_str() {
            // Guardian (existing)
            "risk" | "r" => Self::parse_risk(args),
            "tick" | "t" => Self::Tick,
            "status" | "s" => Self::Status,
            "reset" => Self::Reset,
            "originator" | "o" => Self::parse_originator(args),

            // Signal detection
            "signal" | "sig" => Self::parse_contingency(args, "signal"),
            "prr" => Self::parse_contingency(args, "prr"),
            "ror" => Self::parse_contingency(args, "ror"),
            "ic" => Self::parse_contingency(args, "ic"),
            "ebgm" => Self::parse_contingency(args, "ebgm"),

            // Monitoring
            "health" | "mon" => Self::Health,
            "alerts" => Self::Alerts {
                severity: args.first().map(|s| s.to_string()),
            },
            "sensors" => Self::Sensors,
            "montick" | "mt" => Self::MonitorTick,

            // Patient safety
            "triage" | "tr" => Self::parse_triage(args),
            "priority" | "pri" => Self::parse_priority(args),
            "escalation" | "esc" => Self::parse_escalation(args),

            // Energy
            "energy" | "en" => Self::parse_energy(args),
            "decide" | "dec" => Self::parse_decide(args),

            // Meta
            "help" | "h" | "?" => Self::Help,
            "exit" | "quit" | "q" => Self::Exit,

            other => Self::Unknown(other.to_string()),
        }
    }

    fn parse_risk(args: &[&str]) -> Self {
        if args.len() < 7 {
            return Self::Unknown(
                "Usage: risk <drug> <event> <prr> <ror_lower> <ic025> <eb05> <n>".into(),
            );
        }

        let drug = args[0].trim();
        let event = args[1].trim();
        if drug.is_empty() {
            return Self::Unknown("Drug name cannot be empty".into());
        }
        if event.is_empty() {
            return Self::Unknown("Event name cannot be empty".into());
        }

        let n: u64 = match args[6].parse::<u64>() {
            Ok(v) => v,
            Err(_) => return Self::Unknown(format!("Invalid n: {}", args[6])),
        };

        let context = RiskContext {
            drug: drug.to_string(),
            event: event.to_string(),
            prr: args[2].parse().unwrap_or(0.0),
            ror_lower: args[3].parse().unwrap_or(0.0),
            ic025: args[4].parse().unwrap_or(0.0),
            eb05: args[5].parse().unwrap_or(0.0),
            n,
            originator: OriginatorType::Tool,
        };

        Self::Risk(context)
    }

    fn parse_originator(args: &[&str]) -> Self {
        let type_str = args.first().copied().unwrap_or("tool");
        Self::Originator(type_str.to_string())
    }

    fn parse_contingency(args: &[&str], cmd_name: &str) -> Self {
        if args.len() < 4 {
            return Self::Unknown(format!(
                "Usage: {cmd_name} <a> <b> <c> <d>  (2x2 contingency table)"
            ));
        }

        let parse_u64 = |s: &str, label: &str| -> Result<u64, String> {
            s.parse::<u64>()
                .map_err(|_| format!("Invalid {label}: {s}"))
        };

        let a = match parse_u64(args[0], "a") {
            Ok(v) => v,
            Err(e) => return Self::Unknown(e),
        };
        let b = match parse_u64(args[1], "b") {
            Ok(v) => v,
            Err(e) => return Self::Unknown(e),
        };
        let c = match parse_u64(args[2], "c") {
            Ok(v) => v,
            Err(e) => return Self::Unknown(e),
        };
        let d = match parse_u64(args[3], "d") {
            Ok(v) => v,
            Err(e) => return Self::Unknown(e),
        };

        match cmd_name {
            "signal" => Self::Signal { a, b, c, d },
            "prr" => Self::Prr { a, b, c, d },
            "ror" => Self::Ror { a, b, c, d },
            "ic" => Self::Ic { a, b, c, d },
            "ebgm" => Self::Ebgm { a, b, c, d },
            _ => Self::Unknown(format!("Unknown signal command: {cmd_name}")),
        }
    }

    fn parse_triage(args: &[&str]) -> Self {
        match args.first() {
            Some(s) => Self::Triage {
                seriousness: s.to_string(),
            },
            None => Self::Unknown(
                "Usage: triage <seriousness>  (fatal, life-threatening, disability, hospitalization, congenital, medical, nonserious)"
                    .into(),
            ),
        }
    }

    fn parse_priority(args: &[&str]) -> Self {
        if args.len() < 2 {
            return Self::Unknown(
                "Usage: priority <level_a> <level_b>  (p0-p5 or safety, signal, regulatory, quality, operational, cost)"
                    .into(),
            );
        }
        Self::Priority {
            a: args[0].to_string(),
            b: args[1].to_string(),
        }
    }

    fn parse_escalation(args: &[&str]) -> Self {
        match args.first() {
            Some(s) => Self::Escalation {
                seriousness: s.to_string(),
            },
            None => Self::Unknown(
                "Usage: escalation <seriousness>  (fatal, life-threatening, disability, hospitalization, congenital, medical, nonserious)"
                    .into(),
            ),
        }
    }

    fn parse_energy(args: &[&str]) -> Self {
        match args.first() {
            Some(s) => match s.parse::<u64>() {
                Ok(budget) => Self::Energy { budget },
                Err(_) => Self::Unknown(format!("Invalid budget: {s}")),
            },
            None => Self::Unknown("Usage: energy <budget>  (token budget as integer)".into()),
        }
    }

    fn parse_decide(args: &[&str]) -> Self {
        if args.len() < 3 {
            return Self::Unknown(
                "Usage: decide <budget> <cost> <value>  (e.g. decide 10000 500 3.5)".into(),
            );
        }
        let budget = match args[0].parse::<u64>() {
            Ok(v) => v,
            Err(_) => return Self::Unknown(format!("Invalid budget: {}", args[0])),
        };
        let cost = match args[1].parse::<u64>() {
            Ok(v) => v,
            Err(_) => return Self::Unknown(format!("Invalid cost: {}", args[1])),
        };
        let value = match args[2].parse::<f64>() {
            Ok(v) => v,
            Err(_) => return Self::Unknown(format!("Invalid value: {}", args[2])),
        };
        Self::Decide {
            budget,
            cost,
            value,
        }
    }
}
