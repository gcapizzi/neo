use anyhow::Result;
use clap::{builder::ValueParser, Arg, Command};
use std::path::PathBuf;

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("neo")
        .version("0.1")
        .subcommand(
            Command::new("push").arg(Arg::new("path").value_parser(ValueParser::path_buf())),
        )
        .subcommand(Command::new("list"))
        .get_matches();

    match matches.subcommand() {
        Some(("list", _)) => list(),
        Some(("push", m)) => push(m.get_one::<PathBuf>("path").unwrap()),
        _ => unreachable!(),
    }
}

fn client() -> Result<neo::Client> {
    Ok(neo::Client::new(std::env::var("NEOCITIES_API_KEY")?))
}

fn list() -> Result<()> {
    let files = client()?.list()?;
    for file in files {
        println!("{} {}", file.sha1_hash.unwrap_or(String::new()), file.path)
    }
    Ok(())
}

fn push(p: &PathBuf) -> Result<()> {
    client()?.push(p)?;
    Ok(())
}
