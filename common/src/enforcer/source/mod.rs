use crate::enforcer::dependency::Dependency;
use crate::highlight::Range;
use async_trait::async_trait;
use url::Url;

pub mod cargo;
mod detect;
pub mod maven;
pub mod sbom;

pub use detect::AutoSource;

/// A source of dependencies
#[async_trait]
pub trait Source: Send {
    /// Scan a source for dependencies
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>>;

    /// Find the range to highlight for a specified dependency.
    fn highlight(&self, dependency: &Dependency) -> anyhow::Result<(Url, Range)>;
}
