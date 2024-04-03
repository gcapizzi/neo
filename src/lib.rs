use camino::Utf8Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct ListRes {
    files: Vec<File>,
}

#[derive(Deserialize)]
struct ErrorRes {
    message: String,
}

#[derive(Deserialize)]
pub struct File {
    pub path: String,
    pub is_directory: bool,
    pub size: Option<usize>,
    pub updated_at: String,
    pub sha1_hash: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}: {1}")]
    Api(u16, String),
    #[error("{0}")]
    Transport(String),
    #[error("path {0} is invalid")]
    Path(String),
    #[error(transparent)]
    Json(#[from] std::io::Error),
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Status(status, response) => {
                if let Ok(res) = response.into_json::<ErrorRes>() {
                    Error::Api(status, res.message)
                } else {
                    Error::Api(status, String::new())
                }
            }
            ureq::Error::Transport(t) => Error::Transport(format!("{}", t)),
        }
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub struct Client {
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Client {
        Client { api_key }
    }

    pub fn list(&self) -> Result<Vec<File>> {
        let res: ListRes = ureq::get("https://neocities.org/api/list")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()?
            .into_json()?;
        Ok(res.files)
    }

    pub fn push<'a, I>(&self, entries: I) -> Result<()>
    where
        I: IntoIterator<Item = (&'a Utf8Path, &'a Utf8Path)>,
    {
        let mut m = multipart::client::lazy::Multipart::new();
        for (name, content) in entries {
            m.add_file(name.to_string(), content.to_string());
        }
        let mdata = m.prepare().unwrap();
        let content_type = format!("multipart/form-data; boundary={}", mdata.boundary());

        ureq::post("https://neocities.org/api/upload")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", &content_type)
            .send(mdata)?;

        Ok(())
    }

    pub fn delete<'a, I>(&self, paths: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a Utf8Path>,
    {
        let filenames = paths
            .into_iter()
            .map(|p| ("filenames[]", p.as_str()))
            .collect::<Vec<_>>();
        ureq::post("https://neocities.org/api/delete")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_form(&filenames)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_fs::fixture::{FileWriteStr, NamedTempFile};
    use rand::distributions::{Alphanumeric, DistString};

    #[test]
    fn pushing_listing_and_deleting_files() -> Result<()> {
        let client = super::Client::new(std::env::var("NEOCITIES_API_KEY")?);

        let (file1, file1_sha) = random_txt("file1.txt")?;
        let (file2, file2_sha) = random_txt("file2.txt")?;
        let (file3, file3_sha) = random_txt("file3.txt")?;

        client.push(vec![
            ("up1/file.txt".into(), file1.path().try_into()?),
            ("up2/file.txt".into(), file2.path().try_into()?),
            ("up3/file.txt".into(), file3.path().try_into()?),
        ])?;

        let files = client.list()?;

        let found_file1 = files.iter().find(|f| f.path == "up1/file.txt").unwrap();
        let found_file2 = files.iter().find(|f| f.path == "up2/file.txt").unwrap();
        let found_file3 = files.iter().find(|f| f.path == "up3/file.txt").unwrap();

        assert_eq!(Some(file1_sha), found_file1.sha1_hash);
        assert_eq!(Some(file2_sha), found_file2.sha1_hash);
        assert_eq!(Some(file3_sha), found_file3.sha1_hash);

        client.delete(vec!["up1".into(), "up2".into(), "up3".into()])?;

        Ok(())
    }

    fn random_txt(filename: &str) -> Result<(NamedTempFile, String)> {
        let file = NamedTempFile::new(filename)?;
        let content = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        file.write_str(&content)?;
        let sha1 = sha1_smol::Sha1::from(content).hexdigest();
        Ok((file, sha1))
    }
}
