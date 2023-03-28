use crate::enforcer::{source::Source, Dependency};
use crate::highlight::Range;
use crate::utils::projects::CARGO_FILE;
use anyhow::anyhow;
use async_trait::async_trait;
use cargo_lock::package::Package;
use cargo_lock::Lockfile;
use std::path::PathBuf;
use url::Url;

fn package_to_purl(package: Package) -> Option<Dependency> {
    let name = package.name;
    let version = package.version;

    // the package may have some dependencies, but all the transiant dependencies are flattened
    // in the cargo lockfile so we skip them.
    // However it may be useful later to highlight the source dependency when finding an issue.
    // package.dependencies -> Vec<Dependency>
    let purl = Url::parse(format!("pkg:cargo/{name}@{version}").as_str()).unwrap();

    // todo : add some more information such as git dependencies, patches, custom registry
    // let source = package.source;
    // if let Some(source) = source &&
    //     let SourceKind::Git(git_ref) = source.kind()
    //     {
    //        purl.query_pairs_mut().append_pair("source", format!("{:?}",git_ref).as_str());
    // }

    Some(Dependency { purl })
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
        // find the project root, as the lockfile is not always along the `Cargo.toml` file.
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&self.root.join(CARGO_FILE))
            .exec()?;

        let lockfile_path = metadata.workspace_root.join("Cargo.lock");
        let lockfile = Lockfile::load(lockfile_path)?;

        Ok(lockfile
            .packages
            .into_iter()
            .filter_map(package_to_purl)
            .collect::<Vec<Dependency>>())
    }

    fn highlight(&self, _dependency: &Dependency) -> anyhow::Result<(Url, Range)> {
        Ok((
            Url::from_file_path(self.root.join(CARGO_FILE))
                .map_err(|()| anyhow!("Failed to build path URI"))?,
            Range::default(),
        ))
    }
}
