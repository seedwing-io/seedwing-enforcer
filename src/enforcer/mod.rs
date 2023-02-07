mod dependency;
pub use dependency::*;

pub mod seedwing;
pub mod source;

#[derive(Clone, Debug)]
pub enum Outcome {
    Ok,
    Rejected(String),
}
