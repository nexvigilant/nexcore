//! Secret Scanner - MINIMAL
//!
//! Blocks writes containing secrets. Absolute minimum overhead.

fn main() {
    let mut buf = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).is_err() {
        println!("{{}}");
        return;
    }

    // Fast path: check if content might have secrets (quick heuristics)
    let lower = buf.to_ascii_lowercase();
    let might_have_secrets = lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("private_key")
        || lower.contains("aws_")
        || lower.contains("ghp_")
        || lower.contains("sk-")
        || lower.contains("bearer ")
        || buf.contains("AKIA"); // AWS keys are uppercase

    if !might_have_secrets {
        println!("{{}}");
        return;
    }

    // Parse JSON to check actual content
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&buf) {
        let content = v
            .get("tool_input")
            .and_then(|t| t.get("content").or(t.get("new_string")))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        // Check for obvious secret patterns in actual content
        if has_secret_pattern(content) {
            eprintln!("BLOCKED: Potential secret/credential detected");
            std::process::exit(2);
        }
    }

    println!("{{}}");
}

fn has_secret_pattern(content: &str) -> bool {
    // AWS keys: AKIA followed by 16+ alphanumeric chars
    if content.contains("AKIA") {
        return true;
    }

    // Private keys: -----BEGIN
    if content.contains("-----BEGIN") {
        return true;
    }

    // GitHub PAT: ghp_ followed by 36 chars
    if content.contains("ghp_") {
        return true;
    }

    // OpenAI: sk- followed by many chars
    if content.contains("sk-") && content.len() > 40 {
        for line in content.lines() {
            if line.contains("sk-")
                && line
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .count()
                    > 40
            {
                return true;
            }
        }
    }

    // Generic: key = "long_base64_looking_string"
    for line in content.lines() {
        let line_lower = line.to_ascii_lowercase();
        if (line_lower.contains("api_key")
            || line_lower.contains("secret")
            || line_lower.contains("token"))
            && (line.contains("\"") || line.contains("'"))
        {
            // Check if value looks like a real secret (long alphanumeric)
            if let Some(eq_pos) = line.find('=') {
                let value_part = &line[eq_pos..];
                let alphanum_count = value_part.chars().filter(|c| c.is_alphanumeric()).count();
                if alphanum_count > 20 {
                    return true;
                }
            }
        }
    }

    false
}
