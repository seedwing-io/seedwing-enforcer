use seedwing_policy_engine::value;
use seedwing_policy_engine::value::serde::to_value;
use seedwing_policy_engine::value::RuntimeValue;
use std::fmt::{Display, Formatter};
use url::Url;

/// The internal representation of a dependency
///
/// Currently this is exactly a Package URL. However, we could (and should) add additional
/// information (like scope: dev, build, or regular dependency).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub purl: Url,
}

impl TryFrom<Dependency> for RuntimeValue {
    type Error = value::serde::Error;

    fn try_from(value: Dependency) -> Result<Self, Self::Error> {
        to_value(&value)
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.purl)
    }
}
