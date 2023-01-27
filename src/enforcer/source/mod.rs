use crate::enforcer::dependency::Dependency;
use async_trait::async_trait;

pub mod maven;
pub mod sbom;

/// A source of dependencies
#[async_trait]
pub trait Source: Send {
    /// Scan a source for dependencies
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>>;
}
