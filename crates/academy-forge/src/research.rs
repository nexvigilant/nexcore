use nexcore_perplexity::prelude::*;
use crate::error::{ForgeError, ForgeResult};

pub async fn research_topic(
    topic: &str,
    domain: Option<&str>,
) -> ForgeResult<ResearchResult> {
    let client = PerplexityClient::from_env()
        .map_err(|e| ForgeError::ResearchError(format!("Failed to initialize Perplexity client: {e}")))?;
        
    let query = if let Some(d) = domain {
        format!("Research the topic '{}' specifically within the context of the '{}' domain.", topic, d)
    } else {
        format!("Research the topic '{}' thoroughly.", topic)
    };

    let result = nexcore_perplexity::research::research_regulatory(
        &client,
        &query,
        Some(SearchRecency::Month),
        Some(Model::SonarPro),
    ).await.map_err(|e| ForgeError::ResearchError(format!("Research failed: {e}")))?;

    Ok(result)
}
