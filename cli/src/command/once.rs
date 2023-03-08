use anyhow::bail;
use seedwing_enforcer_common::enforcer::seedwing::Enforcer;
use seedwing_enforcer_common::enforcer::source::maven::MavenSource;
use seedwing_enforcer_common::enforcer::source::Source;
use seedwing_enforcer_common::utils::pool::Pool;
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
#[command(about = "Scan dependencies once", allow_external_subcommands = true)]
pub struct Once {
    #[arg(short, long)]
    source: Option<PathBuf>,
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

impl Once {
    // todo change printlns to a logger ?
    // todo enforcer config option to not wrap rationale in HTML
    pub async fn run(self) -> anyhow::Result<()> {
        // ../target/debug/senf once --source ../common/test-data/pom1.xml

        let pom = MavenSource::new(dir_path(self.source.clone()));
        let dependencies = pom.scan().await;
        if let Err(e) = dependencies {
            bail!("Error: failed scanning dependencies: {:?}", e);
        }

        let mut enforcer = Enforcer::new(dir_path(Some(self.config.clone())), Pool::new()).await;
        enforcer.configure().await;

        let diag = enforcer.diagnostics().await;
        if !diag.is_empty() {
            for (path, issue) in diag {
                println!("{}", path.to_string_lossy());
                for i in issue {
                    println!("\t - {}", i.message)
                }
            }
            bail!("Error: invalid enforcer configuration.");
        }

        match enforcer.eval(dependencies.unwrap()).await {
            Ok(scan) => {
                let mut error = false;
                println!("Scan result:");
                for (dep, outcome) in scan {
                    println!("{} => {}", dep, outcome);

                    if !outcome.is_failed() {
                        error = true;
                    }
                }
                if error {
                    bail!("At least one dependency does not satisfy policies");
                }
            }
            Err(e) => bail!("Error while scanning dependencies : {:?}", e),
        }

        Ok(())
    }
}

// todo allow providing full path to files and not assume file names
fn dir_path(path: Option<PathBuf>) -> PathBuf {
    let path = path.unwrap_or(PathBuf::from("./"));

    if path.is_file() {
        path.parent().unwrap().to_path_buf()
    } else {
        path.to_path_buf()
    }
}
