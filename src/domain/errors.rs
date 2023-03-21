use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error(transparent)]
    Git(GitError),

    #[error(transparent)]
    UserInput(UserInputError),

    #[error("Invalid Configuration: {message}")]
    Configuration {
        message: String,
        source: anyhow::Error,
    },

    #[error(transparent)]
    PersistError(PersistError),

    #[error("Validation error")]
    ValidationError { message: String },
}

#[derive(Error, Debug)]
pub enum UserInputError {
    #[error("Missing required {name:?} input")]
    Required { name: String },

    #[error("Invalid command {name:?} found")]
    InvalidCommand { name: String },

    #[error("Input prompt cancelled by user")]
    Cancelled,

    #[error("Invalid user input {name:?} found")]
    Validation { name: String },
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to read a git provided value")]
    Read,

    #[error("Failed to write to git")]
    Write,
}

#[derive(Error, Debug)]
pub enum PersistError {
    #[error("Invalid store configuration")]
    Configuration,

    #[error("Persisted {name:?} has been corrupted or is out of date")]
    Corrupted {
        name: String,
        source: Option<anyhow::Error>,
    },

    #[error("Requested {name} not found in persisted store")]
    NotFound { name: String },

    #[error("Failed to persist or retrieve {name:?}")]
    Validation { name: String, source: anyhow::Error },

    #[error("Unknown error occurred while connecting persisted store")]
    Unknown(anyhow::Error),
}
