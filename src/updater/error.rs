use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    Http(ureq::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    SemVer(semver::Error),
    Zip(zip::result::ZipError),
    Release(String),
    Update(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::SemVer(e) => write!(f, "version error: {e}"),
            Self::Zip(e) => write!(f, "zip error: {e}"),
            Self::Release(msg) => write!(f, "{msg}"),
            Self::Update(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for UpdateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::SemVer(e) => Some(e),
            Self::Zip(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ureq::Error> for UpdateError {
    fn from(e: ureq::Error) -> Self {
        Self::Http(e)
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for UpdateError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<semver::Error> for UpdateError {
    fn from(e: semver::Error) -> Self {
        Self::SemVer(e)
    }
}

impl From<zip::result::ZipError> for UpdateError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e)
    }
}
