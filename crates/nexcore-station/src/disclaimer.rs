//! Standard NexVigilant disclaimer for all WebMCP Hub configs.

/// The standard disclaimer appended to all config descriptions.
pub const DISCLAIMER: &str = "\
DISCLAIMER: This WebMCP configuration was developed by NexVigilant, LLC \
and is provided as a community resource to assist AI agents in navigating \
pharmacovigilance tools. NexVigilant is not responsible for, and does not \
officially endorse third-party use of this configuration, and expressly \
disclaims any and all liability for damages of any kind arising out of \
the use, reference to, or reliance upon any information or actions \
performed through this resource. No guarantee is provided that the \
content is correct, accurate, complete, up-to-date, or that the \
underlying site structure has not changed. This tool is for educational \
and professional development purposes only and does not constitute \
medical or regulatory advice.";

/// The standard builder tagline.
pub const TAGLINE: &str = "Built by NexVigilant (https://nexvigilant.com) \
— Empowerment Through Vigilance.";

/// Format a full config description with disclaimer.
pub fn with_disclaimer(description: &str) -> String {
    format!("{description} {TAGLINE} {DISCLAIMER} {TAGLINE}")
}
