use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error(transparent)]
    Git(GitError),

    #[error(transparent)]
    UserInput(UserInputError),

    #[error("Failed to persist")]
    PersistError,

    #[error("Validation error")]
    ValidationError,
}

#[derive(Error, Debug)]
pub enum UserInputError {
    #[error("Input prompt cancelled by user")]
    Cancelled,
    #[error("Invalid user input '{name:?}' found")]
    Validation { name: String },
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to read a git provided value")]
    Read,
    #[error("Failed to write to git")]
    Write,
}
