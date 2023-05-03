
use std::sync::mpsc::RecvError;

use serde::Serialize;

use crate::search::SearchError;

#[derive(Debug)]
pub enum Error {
    SQLiteError(rusqlite::Error),
    IOError(std::io::Error),
    SearchError(SearchError),
    RecvError(RecvError),
    UserCancel
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Error::SQLiteError(err) => serializer.serialize_str(&format!("{:?}", err)),
            Error::IOError(err) => serializer.serialize_str(&format!("{:?}", err)),
            Error::SearchError(err) => serializer.serialize_str(&format!("{:?}", err)),
            Error::RecvError(err) => serializer.serialize_str(&format!("{:?}", err)),
            Error::UserCancel => serializer.serialize_str("User cancel operation"),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::SQLiteError(value)
    }
}

impl From<std::io::Error> for Error{
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<SearchError> for Error {
    fn from(value: SearchError) -> Self {
        Self::SearchError(value)
    }
}
 
impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Self::RecvError(value)
    }
}