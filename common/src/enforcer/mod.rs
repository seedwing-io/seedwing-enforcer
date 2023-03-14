mod dependency;

pub use dependency::*;
use serde::Serialize;
use std::fmt::{Display, Formatter};

pub mod cache;
pub mod seedwing;
pub mod source;

#[derive(Clone, Debug, Serialize)]
pub enum Outcome {
    Ok,
    Rejected(String),
}

impl Outcome {
    pub fn is_failed(&self) -> bool {
        match self {
            Outcome::Ok => false,
            Outcome::Rejected(_) => true,
        }
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Ok => write!(f, "OK ✅"),
            Outcome::Rejected(msg) => write!(f, "❌ unsatisfied\n{}", msg),
        }
    }
}
