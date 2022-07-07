use reqwest::header::InvalidHeaderValue as HeaderError;
use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
use std::fmt;

/// Errors returned the sever.
#[derive(Debug, Default, PartialEq)]
pub struct RzdErrors(Vec<String>);

impl RzdErrors {
    pub fn new(errors: Vec<String>) -> Self {
        RzdErrors(errors)
    }

    /// Performs the conversion into `Vec`
    pub fn to_vec(self) -> Vec<String> {
        self.0
    }
}

impl fmt::Display for RzdErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join("; "))
    }
}

impl StdError for RzdErrors {}

/// The Errors wrapper that may occur.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An unsupported operation was called.
    UnsupportedOperation,
    /// Parsing of the date failed.
    ParseDateError(String),
    /// Parsing of the time failed.
    ParseTimeError(String),
    /// Parsing of the station code failed.
    ParseStationCodeError(String),
    /// A too short query passed.
    TooShortQuery,
    /// An empty number of the train passed.
    EmptyTrainNumber,
    /// The request finished with an error.
    ReqwestError(ReqwestError),
    /// The server returned a broken header.
    ReqwestHeaderError(HeaderError),
    /// Data serialization failed.
    SerializeError(String),
    /// Data deserialization failed.
    DeserializeError(String),
    /// The server is probably overloaded.
    RzdServerOverloaded,
    /// The server returned a bad reply.
    FailRzdResponse,
    /// The server returned an error description.
    RzdError(RzdErrors),
    /// Dummy error by default.
    Empty,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnsupportedOperation => {
                write!(f, "операция не поддерживается")
            }
            Error::ParseDateError(ref s) => {
                debug!("parsing date error: {}", s);
                write!(f, "ошибка преобразования даты")
            }
            Error::ParseTimeError(ref s) => {
                debug!("parsing time error: {}", s);
                write!(f, "ошибка преобразования времени")
            }
            Error::ParseStationCodeError(ref s) => {
                debug!("parsing station code error: {}", s);
                write!(f, "ошибка преобразования кода станции")
            }
            Error::TooShortQuery => {
                write!(f, "передан слишком короткий запрос")
            }
            Error::EmptyTrainNumber => {
                write!(f, "передан некорректный номер поезда")
            }
            Error::ReqwestError(ref e) => {
                error!("{}", e);
                write!(f, "не удалось получить данные с сервера \"РЖД\"")
            }
            Error::ReqwestHeaderError(ref e) => {
                error!("{}", e);
                write!(f, "сервер \"РЖД\" вернул некорректные данные")
            }
            Error::SerializeError(ref e) => {
                error!("{}", e);
                write!(f, "не удалось упаковать данные")
            }
            Error::DeserializeError(ref e) => {
                error!("{}", e);
                write!(f, "не удалось распаковать данные")
            }
            Error::RzdServerOverloaded => {
                write!(
                    f,
                    "удаленный сервер перегружен, измените запрос или попробуйте позднее"
                )
            }
            Error::FailRzdResponse => {
                write!(f, "сервер \"РЖД\" вернул некорректные данные")
            }
            Error::RzdError(ref e) => e.fmt(f),
            Error::Empty => {
                write!(f, "ошибок нет")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::ReqwestError(ref e) => Some(e),
            Error::ReqwestHeaderError(ref e) => Some(e),
            Error::RzdError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Error {
        Error::ReqwestError(error)
    }
}

impl From<HeaderError> for Error {
    fn from(error: HeaderError) -> Error {
        Error::ReqwestHeaderError(error)
    }
}

impl From<RzdErrors> for Error {
    fn from(error: RzdErrors) -> Error {
        Error::RzdError(error)
    }
}
