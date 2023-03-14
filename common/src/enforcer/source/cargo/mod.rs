use crate::enforcer::{source::Source, Dependency};
use crate::highlight::Range;
use anyhow::anyhow;
use async_trait::async_trait;
use std::path::PathBuf;
use url::Url;

fn cargo_to_purl(name: String, dep: cargo_toml::Dependency) -> Option<Dependency> {
    match dep {
        cargo_toml::Dependency::Simple(version) => Some(Dependency {
            purl: Url::parse(format!("pkg:cargo/{name}@{version}").as_str()).unwrap(),
        }),
        cargo_toml::Dependency::Detailed(detail) => {
            // if there is no version provided we can't build a compliant package url
            detail.version.map(|version| Dependency {
                purl: Url::parse(format!("pkg:cargo/{}@{}", name, version).as_str()).unwrap(),
            })
        }
        cargo_toml::Dependency::Inherited(_) => unimplemented!(),
    }
}

pub struct CargoSource {
    root: PathBuf,
}

impl CargoSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[async_trait]
impl Source for CargoSource {
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>> {
        let manifest = cargo_toml::Manifest::from_path(self.root.join("Cargo.toml"))?;

        // let content = fs::read_to_string(self.root.join("Cargo.toml"))?;
        // let manifest = cargo_toml::Manifest::from_str(&content)?;

        Ok(manifest
            .dependencies
            .into_iter()
            .filter_map(|(n, d)| cargo_to_purl(n, d))
            .collect::<Vec<Dependency>>())
    }

    fn highlight(&self, _dependency: &Dependency) -> anyhow::Result<(Url, Range)> {
        Ok((
            Url::from_file_path(self.root.join("Cargo.toml"))
                .map_err(|()| anyhow!("Failed to build path URI"))?,
            Range::default(),
        ))
    }
}
