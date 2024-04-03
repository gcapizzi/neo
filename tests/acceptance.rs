use anyhow::Result;
use assert_cmd::Command;
use assert_fs::fixture::{FileWriteStr, NamedTempFile};
use predicates::prelude::*;
use rand::distributions::{Alphanumeric, DistString};

#[test]
fn it_can_upload_a_file_and_delete_it() -> Result<()> {
    let (file1, file1_sha) = random_txt("file1.txt")?;
    let (file2, file2_sha) = random_txt("file2.txt")?;
    let (file3, file3_sha) = random_txt("file3.txt")?;

    Command::cargo_bin("neo")?
        .arg("push")
        .arg(file1.path())
        .arg(file2.path())
        .arg(file3.path())
        .assert()
        .try_success()?;

    Command::cargo_bin("neo")?
        .arg("list")
        .assert()
        .try_success()?
        .try_stdout(predicate::str::contains(format!("{} file1.txt", file1_sha)))?
        .try_stdout(predicate::str::contains(format!("{} file2.txt", file2_sha)))?
        .try_stdout(predicate::str::contains(format!("{} file3.txt", file3_sha)))?;

    Command::cargo_bin("neo")?
        .arg("delete")
        .arg("file1.txt")
        .arg("file2.txt")
        .arg("file3.txt")
        .assert()
        .try_success()?;

    Command::cargo_bin("neo")?
        .arg("list")
        .assert()
        .try_success()?
        .try_stdout(predicate::str::contains("file1.txt").not())?
        .try_stdout(predicate::str::contains("file2.txt").not())?
        .try_stdout(predicate::str::contains("file3.txt").not())?;

    Ok(())
}

fn random_txt(filename: &str) -> Result<(NamedTempFile, String)> {
    let file = NamedTempFile::new(filename)?;
    let content = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    file.write_str(&content)?;
    let sha1 = sha1_smol::Sha1::from(content).hexdigest();
    Ok((file, sha1))
}
