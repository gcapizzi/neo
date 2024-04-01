use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{value_parser, Arg, Command};

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("neo")
        .version("0.1")
        .subcommand(
            Command::new("push").arg(Arg::new("path").value_parser(value_parser!(Utf8PathBuf))),
        )
        .subcommand(Command::new("list"))
        .get_matches();

    match matches.subcommand() {
        Some(("list", _)) => list(),
        Some(("push", m)) => push(&m.get_one::<Utf8PathBuf>("path").unwrap()),
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

fn push(p: &Utf8PathBuf) -> Result<()> {
    client()?.push(p)?;
    Ok(())
}
