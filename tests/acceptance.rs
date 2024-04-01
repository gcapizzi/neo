use anyhow::Result;
use assert_cmd::Command;
use assert_fs::fixture::{FileWriteStr, NamedTempFile};
use predicates::prelude::*;
use rand::distributions::{Alphanumeric, DistString};

#[test]
fn it_can_upload_a_file() -> Result<()> {
    let file = NamedTempFile::new("file.txt")?;
    let content = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    file.write_str(&content)?;
    let sha1 = sha1_smol::Sha1::from(content).hexdigest();

    Command::cargo_bin("neo")?
        .arg("push")
        .arg(file.path())
        .assert()
        .try_success()?;

    Command::cargo_bin("neo")?
        .arg("list")
        .assert()
        .try_success()?
        .try_stdout(predicate::str::contains(format!("{} file.txt", sha1)))?;

    Ok(())
}
