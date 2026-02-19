// ========================================================================
// GCloud Tools (19)
// ========================================================================

#[tool(description = "List authenticated gcloud accounts.")]
pub async fn gcloud_auth_list(&self) -> Result<CallToolResult, McpError> {
    tools::gcloud::auth_list().await
}

#[tool(description = "List current gcloud configuration (project, account, region, zone).")]
pub async fn gcloud_config_list(&self) -> Result<CallToolResult, McpError> {
    tools::gcloud::config_list().await
}

#[tool(description = "Get a specific gcloud configuration property.")]
pub async fn gcloud_config_get(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudConfigGetParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::config_get(&params.property).await
}

#[tool(description = "Set a gcloud configuration property.")]
pub async fn gcloud_config_set(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudConfigSetParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::config_set(&params.property, &params.value).await
}

#[tool(description = "List all accessible GCP projects.")]
pub async fn gcloud_projects_list(&self) -> Result<CallToolResult, McpError> {
    tools::gcloud::projects_list().await
}

#[tool(description = "Get details about a specific GCP project.")]
pub async fn gcloud_projects_describe(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudProjectParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::projects_describe(&params.project_id).await
}

#[tool(description = "Get IAM policy for a GCP project.")]
pub async fn gcloud_projects_get_iam_policy(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudProjectParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::projects_get_iam_policy(&params.project_id).await
}

#[tool(description = "List secrets in Secret Manager.")]
pub async fn gcloud_secrets_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::secrets_list(params.project.as_deref()).await
}

#[tool(description = "Access a secret version's value from Secret Manager.")]
pub async fn gcloud_secrets_versions_access(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudSecretsAccessParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::secrets_versions_access(
        &params.secret_name,
        &params.version,
        params.project.as_deref()
    ).await
}

#[tool(description = "List Cloud Storage buckets.")]
pub async fn gcloud_storage_buckets_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::storage_buckets_list(params.project.as_deref()).await
}

#[tool(description = "List objects in a Cloud Storage path.")]
pub async fn gcloud_storage_ls(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudStoragePathParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::storage_ls(&params.path).await
}

#[tool(description = "Copy files to/from Cloud Storage.")]
pub async fn gcloud_storage_cp(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudStorageCpParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::storage_cp(&params.source, &params.destination, params.recursive).await
}

#[tool(description = "List Compute Engine instances.")]
pub async fn gcloud_compute_instances_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudComputeInstancesParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::compute_instances_list(params.project.as_deref(), params.zone.as_deref()).await
}

#[tool(description = "List Cloud Run services.")]
pub async fn gcloud_run_services_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudServiceListParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::run_services_list(params.project.as_deref(), params.region.as_deref()).await
}

#[tool(description = "Get details about a Cloud Run service.")]
pub async fn gcloud_run_services_describe(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudServiceDescribeParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::run_services_describe(
        &params.name,
        &params.region,
        params.project.as_deref()
    ).await
}

#[tool(description = "List Cloud Functions.")]
pub async fn gcloud_functions_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudServiceListParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::functions_list(params.project.as_deref(), params.region.as_deref()).await
}

#[tool(description = "List service accounts.")]
pub async fn gcloud_iam_service_accounts_list(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::iam_service_accounts_list(params.project.as_deref()).await
}

#[tool(description = "Read log entries from Cloud Logging.")]
pub async fn gcloud_logging_read(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudLoggingReadParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::logging_read(&params.filter, params.limit, params.project.as_deref()).await
}

#[tool(
    description = "Run an arbitrary gcloud command (with safety checks for destructive operations)."
)]
pub async fn gcloud_run_command(
    &self,
    Parameters(params): Parameters<params::gcloud::GcloudRunCommandParams>
) -> Result<CallToolResult, McpError> {
    tools::gcloud::run_command(&params.command, params.timeout).await
}

// ========================================================================
// Wolfram Alpha Tools (19)
// ========================================================================

#[tool(
    description = "Query Wolfram Alpha's computational knowledge engine with full results. Returns comprehensive information organized in pods. Use for: math, science, geography, history, linguistics, music, astronomy, engineering, medicine, finance, sports, food/nutrition, and more. Examples: 'integrate x^2 from 0 to 1', 'distance Earth to Mars', 'GDP of France vs Germany', 'ISS location', 'weather in Tokyo'"
)]
pub async fn wolfram_query(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframQueryParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::query(params).await
}

#[tool(
    description = "Get a concise, single-line answer from Wolfram Alpha. Best for quick facts, simple calculations, and direct questions. More efficient than full query when you just need the answer. Examples: 'population of Japan', 'boiling point of water', '100 miles in km', 'how many days until Christmas'"
)]
pub async fn wolfram_short_answer(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframShortParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::short_answer(params).await
}

#[tool(
    description = "Get a natural language answer suitable for speaking aloud. Returns human-readable sentences rather than data tables. Ideal for voice interfaces, accessibility, or conversational responses. Examples: 'What is the speed of light?', 'How far is the moon?', 'What is the derivative of x cubed?'"
)]
pub async fn wolfram_spoken_answer(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframShortParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::spoken_answer(params).await
}

#[tool(
    description = "Perform mathematical calculations with Wolfram Alpha. Supports arithmetic, algebra, calculus, linear algebra, statistics, number theory, discrete math, and more. Examples: 'solve x^2 + 2x - 3 = 0', 'integral of sin(x)cos(x)', 'determinant of {{1,2},{3,4}}', 'sum of 1/n^2 from n=1 to infinity', 'factor 123456789', 'derivative of e^(x^2)'"
)]
pub async fn wolfram_calculate(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::calculate(params).await
}

#[tool(
    description = "Solve math problems with detailed step-by-step explanations. Shows the work and reasoning for educational purposes. Best for: solving equations, derivatives, integrals, limits, simplification, factoring, and algebraic manipulations. Examples: 'solve 2x + 5 = 13 step by step', 'integrate x*e^x step by step', 'factor x^2 - 5x + 6'"
)]
pub async fn wolfram_step_by_step(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::step_by_step(params).await
}

#[tool(
    description = "Generate mathematical plots and graphs. Returns image URL for the visualization. Supports 2D plots, 3D surfaces, parametric curves, implicit plots, etc. Examples: 'plot sin(x) from -2pi to 2pi', '3D plot of x^2 + y^2', 'plot x^2, x^3, x^4 together', 'parametric plot (cos(t), sin(t)) for t from 0 to 2pi'"
)]
pub async fn wolfram_plot(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::plot(params).await
}

#[tool(
    description = "Convert between units of measurement with high precision. Supports all physical units: length, mass, volume, temperature, speed, energy, pressure, area, time, data sizes, currency, and more. Examples: '100 mph to km/h', '72 fahrenheit to celsius', '1 lightyear to km', '500 MB to GB', '1 acre to square meters'"
)]
pub async fn wolfram_convert(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::convert(params).await
}

#[tool(
    description = "Look up comprehensive chemical compound information. Returns molecular formula, structure, properties, safety data, thermodynamic data, and more. Accepts: compound names, molecular formulas, SMILES, CAS numbers. Examples: 'caffeine', 'H2SO4', 'aspirin properties', 'CAS 50-78-2', 'ethanol boiling point'"
)]
pub async fn wolfram_chemistry(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::chemistry(params).await
}

#[tool(
    description = "Query physics constants, formulas, and calculations. Includes mechanics, electromagnetism, thermodynamics, quantum physics, relativity, optics, and astrophysics. Examples: 'speed of light', 'gravitational constant', 'kinetic energy of 10 kg at 5 m/s', 'wavelength of red light', 'escape velocity of Earth', 'Schwarzschild radius of the Sun'"
)]
pub async fn wolfram_physics(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::physics(params).await
}

#[tool(
    description = "Query astronomical data: planets, stars, galaxies, satellites, events. Real-time positions, rise/set times, orbital data, and more. Examples: 'ISS location', 'Mars distance from Earth', 'next solar eclipse', 'Andromeda galaxy', 'sunrise tomorrow in London', 'Jupiter moons', 'Hubble Space Telescope orbit'"
)]
pub async fn wolfram_astronomy(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::astronomy(params).await
}

#[tool(
    description = "Perform statistical calculations and analysis. Includes descriptive statistics, probability distributions, hypothesis testing, regression, and more. Examples: 'mean of {1, 2, 3, 4, 5}', 'standard deviation of {10, 12, 15, 18, 20}', 'normal distribution mean=100 sd=15', 'P(X > 2) for Poisson(3)', 'linear regression {{1,2},{2,4},{3,5},{4,4},{5,5}}'"
)]
pub async fn wolfram_statistics(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::statistics(params).await
}

#[tool(
    description = "Look up real-world data: demographics, economics, geography, etc. Includes countries, cities, companies, historical data, rankings. Examples: 'population of Tokyo', 'GDP of Germany', 'tallest buildings in the world', 'US unemployment rate', 'distance from New York to London', 'area of Texas'"
)]
pub async fn wolfram_data_lookup(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::data_lookup(params).await
}

#[tool(
    description = "Query Wolfram Alpha with a specific interpretation for ambiguous terms. Use when a previous query returned multiple possible interpretations. Examples: query 'Mercury' with assumption for planet vs element, query 'pi' with assumption for mathematical constant vs movie"
)]
pub async fn wolfram_query_with_assumption(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframAssumptionParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::query_with_assumption(params).await
}

#[tool(
    description = "Query Wolfram Alpha with pod filtering to get specific result types. Reduces noise by including only relevant pods or excluding unwanted ones. Common pod IDs: Result, Input, Plot, Solution, Properties, Definition, BasicInformation, Timeline, Notable facts"
)]
pub async fn wolfram_query_filtered(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframFilteredParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::query_filtered(params).await
}

#[tool(
    description = "Get a visual/image result from Wolfram Alpha. Returns a URL to an image containing the full rendered result. Useful for complex visualizations, formulas, and graphical data. Examples: 'anatomy of the heart', 'world map with time zones', 'periodic table', 'human skeleton'"
)]
pub async fn wolfram_image_result(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::image_result(params).await
}

#[tool(
    description = "Calculate dates, times, durations, and time zones. Examples: 'days until December 25 2025', 'time in Tokyo', 'what day was July 4 1776', '90 days from today', 'duration from Jan 1 2020 to today', 'next full moon'"
)]
pub async fn wolfram_datetime(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::datetime(params).await
}

#[tool(
    description = "Look up nutritional information for foods. Returns calories, macros, vitamins, minerals, and more. Examples: 'nutrition facts for apple', '100g chicken breast calories', 'compare pizza vs salad nutrition', 'vitamin C in orange'"
)]
pub async fn wolfram_nutrition(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::nutrition(params).await
}

#[tool(
    description = "Financial calculations and market data. Includes currency conversion, loan calculations, investment growth, stock data, economic indicators. Examples: '500 USD to EUR', 'mortgage payment 300000 at 6% for 30 years', 'compound interest 10000 at 5% for 10 years', 'inflation adjusted 1000 from 1990 to today'"
)]
pub async fn wolfram_finance(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::finance(params).await
}

#[tool(
    description = "Language and word information: definitions, etymology, translations, word frequency, anagrams, rhymes, and more. Examples: 'define ubiquitous', 'etymology of algorithm', 'translate hello to Japanese', 'anagrams of listen', 'words that rhyme with time', 'Scrabble score for quizzify'"
)]
pub async fn wolfram_linguistics(
    &self,
    Parameters(params): Parameters<params::wolfram::WolframBaseParams>
) -> Result<CallToolResult, McpError> {
    tools::wolfram::linguistics(params).await
}

// ========================================================================
// Perplexity AI Search Tools (4)
// ========================================================================

#[tool(
    description = "Search the web using Perplexity AI's Sonar API. Returns search-grounded answers with citations. Supports model selection (sonar=fast, sonar-pro=deep, sonar-deep-research=multi-step), recency filtering (hour/day/week/month), and domain filtering. Set PERPLEXITY_API_KEY env var. Examples: 'latest FDA drug approvals 2026', 'Rust async runtime comparison'"
)]
pub async fn perplexity_search(
    &self,
    Parameters(params): Parameters<params::perplexity::PerplexitySearchParams>
) -> Result<CallToolResult, McpError> {
    tools::perplexity::search(params).await
}

#[tool(
    description = "High-level research routing. Specify use_case: 'general' (open web research), 'competitive' (market/competitor intel with SonarPro), or 'regulatory' (FDA/EMA/ICH/WHO filtered). Each use case applies specialized system prompts and domain filters."
)]
pub async fn perplexity_research(
    &self,
    Parameters(params): Parameters<params::perplexity::PerplexityResearchParams>
) -> Result<CallToolResult, McpError> {
    tools::perplexity::research(params).await
}

#[tool(
    description = "Competitive intelligence search filtered to specified competitor domains. Uses SonarPro model with month recency filter. Provide competitor domain names (e.g., ['competitor1.com', 'competitor2.io']). Focuses on market positioning, pricing, announcements, partnerships."
)]
pub async fn perplexity_competitive(
    &self,
    Parameters(params): Parameters<params::perplexity::PerplexityCompetitiveParams>
) -> Result<CallToolResult, McpError> {
    tools::perplexity::competitive(params).await
}

#[tool(
    description = "Regulatory intelligence search pre-filtered to pharmaceutical/healthcare regulatory domains: fda.gov, ema.europa.eu, ich.org, who.int, clinicaltrials.gov, drugs.com, drugbank.com, pmda.go.jp. Uses SonarPro with month recency. For FDA actions, EMA guidelines, ICH harmonization, WHO reports."
)]
pub async fn perplexity_regulatory(
    &self,
    Parameters(params): Parameters<params::perplexity::PerplexityRegulatoryParams>
) -> Result<CallToolResult, McpError> {
    tools::perplexity::regulatory(params).await
}
