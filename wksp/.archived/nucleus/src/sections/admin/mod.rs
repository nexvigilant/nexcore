//! Admin section — management of all platform resources

mod dashboard;
mod academy;
mod community;
mod content;
mod intelligence;
mod users;
mod settings;
mod ventures;

pub use dashboard::DashboardPage;
pub use academy::AcademyAdminPage;
pub use community::CommunityAdminPage;
pub use content::ContentAdminPage;
pub use intelligence::IntelligenceAdminPage;
pub use users::UsersAdminPage;
pub use settings::SettingsAdminPage;
pub use ventures::VenturesAdminPage;