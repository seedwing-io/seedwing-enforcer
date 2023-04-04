mod dependency;

pub use dependency::*;
use serde::Serialize;

pub mod cache;
pub mod seedwing;
pub mod source;

pub use seedwing_policy_engine::lang::Severity;
pub use seedwing_policy_engine::runtime::Response;

#[derive(Clone, Debug, Serialize)]
pub enum Outcome {
    Ok,
    RejectedHtml(String),
    RejectedRaw(Response),
}

impl Outcome {
    pub fn is_failed(&self) -> bool {
        matches!(self, Outcome::Ok)
    }
}
