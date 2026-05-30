use thiserror::Error;

/// Errors raised by the repository/conversion layer (e.g. decoding a stored
/// admin-action row back into its typed form).
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("invalid action kind: {0}")]
    InvalidActionKind(String),
    #[error("invalid max debt value")]
    InvalidMaxDebtValue,
}
