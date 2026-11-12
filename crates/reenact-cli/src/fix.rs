use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct FixArgs {
    #[arg(long)]
    pub case: PathBuf,
}

pub async fn run(_a: FixArgs) -> Result<()> {
    anyhow::bail!("not implemented yet")
}
