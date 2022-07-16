use tungstenite;
use url;

pub enum Error {
    TungsteniteError(tungstenite::Error),
    IoError(std::io::Error),
    ParseError(url::ParseError),
    NotFoundError,
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Error::TungsteniteError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::ParseError(err)
    }
}
