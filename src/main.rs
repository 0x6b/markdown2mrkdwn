use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use markdown2mrkdwn::Mrkdwn;

#[cfg(feature = "bin")]
#[derive(Debug, Parser)]
struct Args {
    #[arg()]
    path: PathBuf,
}

#[cfg(feature = "bin")]
fn main() -> Result<()> {
    let args = Args::parse();

    print!(
        "{}",
        Mrkdwn::from(read_to_string(args.path)?.as_str())
            .mrkdwnify()?
            .replace("\\\"", "\"")
            .replace("&amp;", "&")
            .replace("\\n", "\n")
    );

    Ok(())
}
