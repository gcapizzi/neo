use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{value_parser, Arg, Command, ArgAction};

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("neo")
        .version("0.1")
        .subcommand(Command::new("list"))
        .subcommand(
            Command::new("push").arg(Arg::new("path").action(ArgAction::Append).value_parser(value_parser!(Utf8PathBuf))),
        )
        .subcommand(
            Command::new("delete").arg(Arg::new("path").value_parser(value_parser!(Utf8PathBuf))),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("list", _)) => list(),
        Some(("push", m)) => push(m.get_many::<Utf8PathBuf>("path").unwrap()),
        Some(("delete", m)) => delete(&m.get_one::<Utf8PathBuf>("path").unwrap()),
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

fn push<'a, I: IntoIterator<Item = &'a Utf8PathBuf>>(paths: I) -> Result<()> {
    let entries = paths
        .into_iter()
        .map(|p| {
            let src = p.as_path();
            let dst: &Utf8Path = p
                .file_name()
                .ok_or(anyhow!("invalid path: {}", p))?
                .try_into()?;
            Ok((dst, src))
        })
        .collect::<Result<Vec<_>>>()?;
    client()?.push(entries)?;
    Ok(())
}

fn delete(f: &Utf8Path) -> Result<()> {
    client()?.delete(f)?;
    Ok(())
}
