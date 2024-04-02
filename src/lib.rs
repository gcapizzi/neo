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

    pub fn list(&self) -> Result<Vec<File>, Error> {
        let res: ListRes = ureq::get("https://neocities.org/api/list")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()?
            .into_json()?;
        Ok(res.files)
    }

    pub fn push(&self, p: &Utf8Path) -> Result<()> {
        let file_name: &str = p.file_name().ok_or(Error::Path(p.to_string()))?;

        let mut m = multipart::client::lazy::Multipart::new();
        m.add_file(file_name, p.to_string());
        let mdata = m.prepare().unwrap();
        let content_type = format!("multipart/form-data; boundary={}", mdata.boundary());

        ureq::post("https://neocities.org/api/upload")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", &content_type)
            .send(mdata)?;

        Ok(())
    }

    pub fn delete(&self, f: &Utf8Path) -> Result<()> {
        ureq::post("https://neocities.org/api/delete")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_form(&[("filenames[]", f.as_str())])?;

        Ok(())
    }
}
