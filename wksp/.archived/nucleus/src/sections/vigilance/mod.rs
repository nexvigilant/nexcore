//! Vigilance section — absorbed from NCOS
//!
//! Signal detection, Guardian homeostasis, PVDSL, Brain, causality assessment.

mod dashboard;
mod signals;
mod guardian;
mod brain;
mod causality;
mod pvdsl;
mod reporting;
mod skills;
mod benefit_risk;
mod settings;

pub use dashboard::DashboardPage;
pub use signals::SignalsPage;
pub use guardian::GuardianPage;
pub use brain::BrainPage;
pub use causality::CausalityPage;
pub use pvdsl::PvdslPage;
pub use reporting::ReportingPage;
pub use skills::SkillsPage;
pub use benefit_risk::BenefitRiskPage;
pub use settings::SettingsPage;