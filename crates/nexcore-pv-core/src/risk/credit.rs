//! # Patient Safety Credit Scoring
//!
//! Pharmacovigilance adaptation of FICO credit scoring methodology for
//! quantifying patient safety risk profiles and drug safety creditworthiness.
//!
//! ## FICO-to-PV Transformation
//!
//! | FICO Concept | PV Adaptation |
//! |--------------|---------------|
//! | Payment History (35%) | Adverse event history |
//! | Credit Utilization (30%) | Drug exposure levels |
//! | Credit History Length (15%) | Duration of drug experience |
//! | Credit Mix (10%) | Diversity of safety data sources |
//! | New Credit (10%) | Recent safety signal investigations |

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

const WEIGHT_SAFETY_HISTORY: f64 = 0.35;
const WEIGHT_EXPOSURE_UTILIZATION: f64 = 0.30;
const WEIGHT_EXPERIENCE_LENGTH: f64 = 0.15;
const WEIGHT_DATA_SOURCE_MIX: f64 = 0.10;
const WEIGHT_RECENT_ACTIVITY: f64 = 0.10;
const MIN_SCORE: f64 = 300.0;
const MAX_SCORE: f64 = 850.0;

/// Risk tier based on overall safety credit score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskTier {
    Exceptional,
    VeryGood,
    Good,
    Fair,
    Poor,
}

impl RiskTier {
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        if score >= 800.0 {
            Self::Exceptional
        } else if score >= 740.0 {
            Self::VeryGood
        } else if score >= 670.0 {
            Self::Good
        } else if score >= 580.0 {
            Self::Fair
        } else {
            Self::Poor
        }
    }
}

/// Impact assessment for a scoring component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Impact {
    Positive,
    Neutral,
    Negative,
}

impl Impact {
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        if score >= 750.0 {
            Self::Positive
        } else if score >= 650.0 {
            Self::Neutral
        } else {
            Self::Negative
        }
    }
}

/// Pregnancy category (FDA classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PregnancyCategory {
    A,
    B,
    C,
    D,
    X,
    N,
}

/// Score trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScoreTrend {
    Improving,
    Stable,
    Declining,
}

/// Safety history component input (35% weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyHistory {
    pub total_reports: u32,
    pub serious_reports: u32,
    pub timely_reporting: f64,
    pub report_quality: f64,
    pub regulatory_actions: u32,
    pub history_length: f64,
}

/// Exposure utilization component input (30% weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureProfile {
    pub current_exposure: f64,
    pub max_safe_exposure: f64,
    pub utilization_rate: f64,
    pub peak_exposure: f64,
    pub exposure_variability: f64,
}

/// Safety experience length component input (15% weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceProfile {
    pub market_experience: f64,
    pub clinical_trial_duration: f64,
    pub post_marketing_studies: u32,
    pub real_world_evidence: f64,
}

/// Data source diversity component input (10% weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceMix {
    pub spontaneous_reports: bool,
    pub clinical_trials: bool,
    pub registries: bool,
    pub literature_reports: bool,
    pub regulatory_databases: bool,
    pub social_media_monitoring: bool,
    pub electronic_health_records: bool,
}

impl DataSourceMix {
    #[must_use]
    pub fn source_count(&self) -> u32 {
        u32::from(self.spontaneous_reports)
            + u32::from(self.clinical_trials)
            + u32::from(self.registries)
            + u32::from(self.literature_reports)
            + u32::from(self.regulatory_databases)
            + u32::from(self.social_media_monitoring)
            + u32::from(self.electronic_health_records)
    }
}

/// Recent safety activity component input (10% weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub recent_signals: u32,
    pub recent_studies: u32,
    pub recent_submissions: u32,
    pub media_attention: f64,
    pub days_since_last_activity: u32,
}

/// Additional risk factors affecting score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    pub pregnancy_category: Option<PregnancyCategory>,
    pub pediatric_use: bool,
    pub geriatric_use: bool,
    pub hepatic_impairment: bool,
    pub renal_impairment: bool,
    pub drug_interactions: u32,
    pub black_box_warning: bool,
    pub rems: bool,
}

/// Complete input for safety credit scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditInput {
    pub safety_history: SafetyHistory,
    pub exposure_profile: ExposureProfile,
    pub experience_profile: ExperienceProfile,
    pub data_source_mix: DataSourceMix,
    pub recent_activity: RecentActivity,
    pub risk_factors: RiskFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScore {
    pub score: f64,
    pub weight: f64,
    pub impact: Impact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactorAnalysis {
    pub critical_flags: Vec<String>,
    pub moderate_flags: Vec<String>,
    pub positive_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreChangePoint {
    pub days_ago: u32,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistory {
    pub previous_score: Option<f64>,
    pub trend: ScoreTrend,
    pub change_points: Vec<ScoreChangePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmarks {
    pub industry_average: f64,
    pub therapeutic_area_average: f64,
    pub similar_drugs_average: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditResult {
    pub overall_score: f64,
    pub risk_tier: RiskTier,
    pub components: CreditComponents,
    pub risk_factors: RiskFactorAnalysis,
    pub recommendations: Vec<String>,
    pub score_history: ScoreHistory,
    pub benchmarks: Benchmarks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditComponents {
    pub safety_history: ComponentScore,
    pub exposure_utilization: ComponentScore,
    pub experience_length: ComponentScore,
    pub data_source_mix: ComponentScore,
    pub recent_activity: ComponentScore,
}

#[inline]
fn clamp_score(score: f64) -> f64 {
    score.clamp(MIN_SCORE, MAX_SCORE)
}

fn calculate_safety_history_score(h: &SafetyHistory) -> ComponentScore {
    let mut score = MAX_SCORE;
    let rate = if h.total_reports > 0 {
        f64::from(h.serious_reports) / f64::from(h.total_reports)
    } else {
        0.0
    };
    if rate > 0.1 {
        score -= 150.0;
    } else if rate > 0.05 {
        score -= 100.0;
    } else if rate > 0.02 {
        score -= 50.0;
    }
    if h.timely_reporting < 0.7 {
        score -= 100.0;
    } else if h.timely_reporting < 0.9 {
        score -= 50.0;
    } else if h.timely_reporting >= 0.95 {
        score += 25.0;
    }
    if h.report_quality < 5.0 {
        score -= 75.0;
    } else if h.report_quality < 7.0 {
        score -= 25.0;
    } else if h.report_quality >= 9.0 {
        score += 25.0;
    }
    if h.regulatory_actions > 5 {
        score -= 200.0;
    } else if h.regulatory_actions > 2 {
        score -= 100.0;
    } else if h.regulatory_actions > 0 {
        score -= 50.0;
    }
    if h.history_length >= 10.0 {
        score += 50.0;
    } else if h.history_length >= 5.0 {
        score += 25.0;
    } else if h.history_length < 1.0 {
        score -= 100.0;
    }
    let score = clamp_score(score);
    ComponentScore {
        score,
        weight: WEIGHT_SAFETY_HISTORY,
        impact: Impact::from_score(score),
    }
}

fn calculate_exposure_utilization_score(e: &ExposureProfile) -> ComponentScore {
    let mut score = MAX_SCORE;
    if e.utilization_rate > 0.9 {
        score -= 150.0;
    } else if e.utilization_rate > 0.7 {
        score -= 100.0;
    } else if e.utilization_rate > 0.5 {
        score -= 50.0;
    } else if e.utilization_rate < 0.1 {
        score += 25.0;
    }
    if e.exposure_variability > 1.0 {
        score -= 100.0;
    } else if e.exposure_variability > 0.5 {
        score -= 50.0;
    } else if e.exposure_variability < 0.2 {
        score += 25.0;
    }
    let peak_ratio = if e.current_exposure > 0.0 {
        e.peak_exposure / e.current_exposure
    } else {
        1.0
    };
    if peak_ratio > 3.0 {
        score += 50.0;
    } else if peak_ratio < 1.1 {
        score -= 75.0;
    }
    let score = clamp_score(score);
    ComponentScore {
        score,
        weight: WEIGHT_EXPOSURE_UTILIZATION,
        impact: Impact::from_score(score),
    }
}

fn calculate_experience_length_score(e: &ExperienceProfile) -> ComponentScore {
    let mut score = MAX_SCORE;
    if e.market_experience >= 20.0 {
        score += 50.0;
    } else if e.market_experience >= 10.0 {
        score += 25.0;
    } else if e.market_experience >= 5.0 {
        score += 10.0;
    } else if e.market_experience < 1.0 {
        score -= 100.0;
    }
    if e.clinical_trial_duration >= 10.0 {
        score += 25.0;
    } else if e.clinical_trial_duration < 2.0 {
        score -= 50.0;
    }
    if e.post_marketing_studies >= 5 {
        score += 25.0;
    } else if e.post_marketing_studies == 0 {
        score -= 25.0;
    }
    if e.real_world_evidence >= 5.0 {
        score += 25.0;
    } else if e.real_world_evidence < 1.0 {
        score -= 50.0;
    }
    let score = clamp_score(score);
    ComponentScore {
        score,
        weight: WEIGHT_EXPERIENCE_LENGTH,
        impact: Impact::from_score(score),
    }
}

fn calculate_data_source_mix_score(s: &DataSourceMix) -> ComponentScore {
    let mut score = 700.0;
    let count = s.source_count();
    if count >= 6 {
        score += 100.0;
    } else if count >= 4 {
        score += 50.0;
    } else if count >= 2 {
        score += 25.0;
    } else {
        score -= 100.0;
    }
    if s.clinical_trials {
        score += 25.0;
    }
    if s.registries {
        score += 25.0;
    }
    if s.electronic_health_records {
        score += 25.0;
    }
    if s.social_media_monitoring {
        score += 15.0;
    }
    let score = clamp_score(score);
    ComponentScore {
        score,
        weight: WEIGHT_DATA_SOURCE_MIX,
        impact: Impact::from_score(score),
    }
}

fn calculate_recent_activity_score(a: &RecentActivity) -> ComponentScore {
    let mut score = 750.0;
    if a.recent_signals > 5 {
        score -= 150.0;
    } else if a.recent_signals > 2 {
        score -= 75.0;
    } else if a.recent_signals == 0 {
        score += 50.0;
    }
    if a.recent_studies > 3 {
        score += 25.0;
    } else if a.recent_studies == 0 {
        score -= 25.0;
    }
    if a.media_attention > 7.0 {
        score -= 100.0;
    } else if a.media_attention > 4.0 {
        score -= 50.0;
    } else if a.media_attention < 2.0 {
        score += 25.0;
    }
    if a.days_since_last_activity > 180 {
        score -= 50.0;
    } else if a.days_since_last_activity < 30 {
        score += 25.0;
    }
    let score = clamp_score(score);
    ComponentScore {
        score,
        weight: WEIGHT_RECENT_ACTIVITY,
        impact: Impact::from_score(score),
    }
}

fn analyze_risk_factors(f: &RiskFactors) -> RiskFactorAnalysis {
    let mut critical = Vec::new();
    let mut moderate = Vec::new();
    let mut positive = Vec::new();
    if f.black_box_warning {
        critical.push("FDA Black Box Warning".into());
    }
    if let Some(cat) = f.pregnancy_category {
        if matches!(cat, PregnancyCategory::D | PregnancyCategory::X) {
            critical.push(format!("Pregnancy Category {cat:?}"));
        }
    }
    if f.rems {
        critical.push("REMS Required".into());
    }
    if f.hepatic_impairment {
        moderate.push("Hepatic Impairment Concerns".into());
    }
    if f.renal_impairment {
        moderate.push("Renal Impairment Concerns".into());
    }
    if f.drug_interactions > 10 {
        moderate.push("Multiple Drug Interactions".into());
    }
    if let Some(cat) = f.pregnancy_category {
        if matches!(cat, PregnancyCategory::A | PregnancyCategory::B) {
            positive.push("Favorable Pregnancy Category".into());
        }
    }
    if f.pediatric_use && f.geriatric_use {
        positive.push("Broad Age Range Approval".into());
    }
    if f.drug_interactions < 3 {
        positive.push("Minimal Drug Interactions".into());
    }
    RiskFactorAnalysis {
        critical_flags: critical,
        moderate_flags: moderate,
        positive_factors: positive,
    }
}

fn generate_recommendations(
    score: f64,
    rf: &RiskFactorAnalysis,
    c: &CreditComponents,
) -> Vec<String> {
    let mut r = Vec::new();
    if score < 580.0 {
        r.push("URGENT: Comprehensive safety review required".into());
    }
    if c.safety_history.impact == Impact::Negative {
        r.push("Improve adverse event reporting timeliness and quality".into());
    }
    if c.exposure_utilization.impact == Impact::Negative {
        r.push("Monitor patient exposure levels more closely".into());
    }
    if c.data_source_mix.impact == Impact::Negative {
        r.push("Diversify safety data sources".into());
    }
    if !rf.critical_flags.is_empty() {
        r.push("Address critical safety flags immediately".into());
    }
    if score >= 750.0 {
        r.push("Maintain current excellent safety profile".into());
    }
    r
}

/// Calculate comprehensive safety credit score (300-850 scale)
#[must_use]
pub fn calculate_safety_credit_score(input: &CreditInput) -> CreditResult {
    let sh = calculate_safety_history_score(&input.safety_history);
    let eu = calculate_exposure_utilization_score(&input.exposure_profile);
    let el = calculate_experience_length_score(&input.experience_profile);
    let ds = calculate_data_source_mix_score(&input.data_source_mix);
    let ra = calculate_recent_activity_score(&input.recent_activity);
    let overall = (sh.score * sh.weight
        + eu.score * eu.weight
        + el.score * el.weight
        + ds.score * ds.weight
        + ra.score * ra.weight)
        .round();
    let components = CreditComponents {
        safety_history: sh,
        exposure_utilization: eu,
        experience_length: el,
        data_source_mix: ds,
        recent_activity: ra,
    };
    let rf = analyze_risk_factors(&input.risk_factors);
    let recommendations = generate_recommendations(overall, &rf, &components);
    CreditResult {
        overall_score: overall,
        risk_tier: RiskTier::from_score(overall),
        components,
        risk_factors: rf,
        recommendations,
        score_history: ScoreHistory {
            previous_score: None,
            trend: ScoreTrend::Stable,
            change_points: Vec::new(),
        },
        benchmarks: Benchmarks {
            industry_average: 680.0,
            therapeutic_area_average: 695.0,
            similar_drugs_average: 710.0,
        },
    }
}

/// Batch scoring for multiple drugs/products
#[must_use]
pub fn batch_safety_credit_scoring(
    inputs: &[(String, CreditInput)],
) -> Vec<(String, CreditResult)> {
    inputs
        .iter()
        .map(|(id, input)| (id.clone(), calculate_safety_credit_score(input)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> CreditInput {
        CreditInput {
            safety_history: SafetyHistory {
                total_reports: 1000,
                serious_reports: 20,
                timely_reporting: 0.95,
                report_quality: 8.5,
                regulatory_actions: 0,
                history_length: 8.0,
            },
            exposure_profile: ExposureProfile {
                current_exposure: 50000.0,
                max_safe_exposure: 100000.0,
                utilization_rate: 0.5,
                peak_exposure: 75000.0,
                exposure_variability: 0.3,
            },
            experience_profile: ExperienceProfile {
                market_experience: 12.0,
                clinical_trial_duration: 5.0,
                post_marketing_studies: 8,
                real_world_evidence: 7.0,
            },
            data_source_mix: DataSourceMix {
                spontaneous_reports: true,
                clinical_trials: true,
                registries: true,
                literature_reports: true,
                regulatory_databases: true,
                social_media_monitoring: false,
                electronic_health_records: true,
            },
            recent_activity: RecentActivity {
                recent_signals: 1,
                recent_studies: 2,
                recent_submissions: 1,
                media_attention: 2.0,
                days_since_last_activity: 15,
            },
            risk_factors: RiskFactors {
                pregnancy_category: None,
                pediatric_use: true,
                geriatric_use: true,
                hepatic_impairment: false,
                renal_impairment: false,
                drug_interactions: 5,
                black_box_warning: false,
                rems: false,
            },
        }
    }

    #[test]
    fn test_score_bounds() {
        let r = calculate_safety_credit_score(&test_input());
        assert!(r.overall_score >= 300.0 && r.overall_score <= 850.0);
    }

    #[test]
    fn test_risk_tier() {
        assert_eq!(RiskTier::from_score(850.0), RiskTier::Exceptional);
        assert_eq!(RiskTier::from_score(740.0), RiskTier::VeryGood);
        assert_eq!(RiskTier::from_score(670.0), RiskTier::Good);
        assert_eq!(RiskTier::from_score(580.0), RiskTier::Fair);
        assert_eq!(RiskTier::from_score(300.0), RiskTier::Poor);
    }

    #[test]
    fn test_good_profile() {
        let r = calculate_safety_credit_score(&test_input());
        assert!(r.overall_score >= 700.0);
    }

    #[test]
    fn test_poor_profile() {
        let mut i = test_input();
        i.safety_history.serious_reports = 200;
        i.safety_history.regulatory_actions = 6;
        i.safety_history.timely_reporting = 0.5;
        i.safety_history.report_quality = 3.0;
        i.safety_history.history_length = 0.5;
        i.exposure_profile.utilization_rate = 0.95;
        i.exposure_profile.exposure_variability = 1.5;
        i.recent_activity.recent_signals = 10;
        i.recent_activity.media_attention = 9.0;
        let r = calculate_safety_credit_score(&i);
        // Poor profile should score in Fair or Poor tier (< 670)
        assert!(r.overall_score < 670.0);
    }

    #[test]
    fn test_critical_flags() {
        let mut i = test_input();
        i.risk_factors.black_box_warning = true;
        i.risk_factors.pregnancy_category = Some(PregnancyCategory::X);
        let r = calculate_safety_credit_score(&i);
        assert!(r.risk_factors.critical_flags.len() >= 2);
    }

    #[test]
    fn test_weights_sum() {
        let sum = WEIGHT_SAFETY_HISTORY
            + WEIGHT_EXPOSURE_UTILIZATION
            + WEIGHT_EXPERIENCE_LENGTH
            + WEIGHT_DATA_SOURCE_MIX
            + WEIGHT_RECENT_ACTIVITY;
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_batch() {
        let inputs = vec![("a".into(), test_input()), ("b".into(), test_input())];
        let r = batch_safety_credit_scoring(&inputs);
        assert_eq!(r.len(), 2);
    }
}
