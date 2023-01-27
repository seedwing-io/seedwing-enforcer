use seedwing_policy_engine::value;
use seedwing_policy_engine::value::serde::to_value;
use seedwing_policy_engine::value::RuntimeValue;
use url::Url;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Dependency {
    pub purl: Url,
}

impl TryFrom<Dependency> for RuntimeValue {
    type Error = value::serde::Error;

    fn try_from(value: Dependency) -> Result<Self, Self::Error> {
        to_value(&value)
    }
}
