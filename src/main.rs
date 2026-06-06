#![cfg(feature = "bin")]

use std::{
    fs::read_to_string,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use markdown2mrkdwn::Mrkdwn;

#[derive(Debug, Parser)]
#[clap(about = "Convert markdown to mrkdwn format and dump it to stdout")]
struct Args {
    /// Path to a markdown file to convert to mrkdwn. If not provided, the content will be read
    /// from stdin.
    #[arg()]
    path: Option<PathBuf>,

    /// Whether to convert markdown to mrkdwn blocks. Defaults to plain mrkdwn.
    #[arg(short, long)]
    blocks: bool,
}

fn main() -> Result<()> {
    let Args { path, blocks } = Args::parse();
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
        if blocks {
            Mrkdwn::from(input.as_str()).blocks_stringify()?
        } else {
            Mrkdwn::from(input.as_str())
                .mrkdwnify()?
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
        }
    );

    Ok(())
}
