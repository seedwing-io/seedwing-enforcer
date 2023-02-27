//! Definitions used over the Language Server Protocol

pub mod commands {
    pub const SHOW_REPORT: &str = "seedwingEnforcer.showReport";
}

pub mod types {
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct Report {
        pub title: String,
        pub html: String,
    }
}
