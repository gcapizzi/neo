use anyhow::{anyhow, Result};
use camino::Utf8Path;
use clap::{Arg, ArgAction, Command};

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("neo")
        .version("0.1")
        .subcommand(Command::new("list"))
        .subcommand(Command::new("push").arg(Arg::new("path").action(ArgAction::Append)))
        .subcommand(Command::new("delete").arg(Arg::new("path").action(ArgAction::Append)))
        .get_matches();

    match matches.subcommand() {
        Some(("list", _)) => list(),
        Some(("push", m)) => push(m.get_many::<String>("path").unwrap()),
        Some(("delete", m)) => delete(m.get_many::<String>("path").unwrap()),
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

fn push<'a, I: IntoIterator<Item = &'a String>>(paths: I) -> Result<()> {
    let entries = paths
        .into_iter()
        .map(|p| {
            let utf_path = Utf8Path::new(p);
            let file_name = utf_path.file_name().ok_or(anyhow!("invalid path: {}", p))?;
            Ok((file_name, utf_path))
        })
        .collect::<Result<Vec<_>>>()?;
    client()?.push(entries)?;
    Ok(())
}

fn delete<'a, I: IntoIterator<Item = &'a String>>(paths: I) -> Result<()> {
    client()?.delete(paths)?;
    Ok(())
}
