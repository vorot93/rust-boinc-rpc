#[derive(PartialEq, Debug)]
pub enum Error {
    ConnectError(String),
    DataParseError(String),
    InvalidPasswordError(String),
    DaemonError(String),
    NullError(String),
    NetworkError(String),
    StatusError(i32),
    AuthError(String),
    InvalidURLError(String),
    AlreadyAttachedError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::NetworkError(format!("{}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::DataParseError(format!("UTF-8 conversion error: {}", e.utf8_error()))
    }
}

impl From<treexml::Error> for Error {
    fn from(e: treexml::Error) -> Self {
        Error::DataParseError(format!("XML error: {}", e))
    }
}
