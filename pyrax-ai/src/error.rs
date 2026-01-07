use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid job state: expected {expected}, got {actual}")]
    InvalidJobState { expected: String, actual: String },

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    #[error("Worker not registered")]
    WorkerNotRegistered,

    #[error("Worker already claimed this job")]
    AlreadyClaimed,

    #[error("Job timeout")]
    JobTimeout,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
