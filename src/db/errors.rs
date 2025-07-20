use crate::errors::AccountingError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub enum RepositoryError {
    InvalidActionKind(String),
    InvalidMaxDebtValue,
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::InvalidActionKind(kind) => {
                write!(f, "Invalid action kind: {}", kind)
            }
            RepositoryError::InvalidMaxDebtValue => write!(f, "Invalid max debt value"),
        }
    }
}

impl Error for RepositoryError {}

impl Into<AccountingError> for RepositoryError {
    fn into(self) -> AccountingError {
        AccountingError::RepositoryError(self)
    }
}
