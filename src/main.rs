use std::{
    fs::read_to_string,
    io::{stdin, Read},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use markdown2mrkdwn::Mrkdwn;

#[cfg(feature = "bin")]
#[derive(Debug, Parser)]
#[clap(about = "Convert markdown to mrkdwn format and dump it to stdout")]
struct Args {
    /// Path to a markdown file to convert to mrkdwn. If not provided, the content will be read
    /// from stdin.
    #[arg()]
    path: Option<PathBuf>,
}

#[cfg(feature = "bin")]
fn main() -> Result<()> {
    let Args { path } = Args::parse();
    let input = match path {
        None => {
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer)?;
            buffer
        }
        Some(p) => read_to_string(p)?,
    };

    print!(
        "{}",
        Mrkdwn::from(&input)
            .mrkdwnify()?
            .replace("\\\"", "\"")
            .replace("&amp;", "&")
            .replace("\\n", "\n")
    );

    Ok(())
}
