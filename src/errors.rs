use crate::db::errors::RepositoryError;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum AccountingError {
    RepositoryError(RepositoryError),
}

impl Display for AccountingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountingError::RepositoryError(err) => err.fmt(f),
        }
    }
}

impl Error for AccountingError {}
