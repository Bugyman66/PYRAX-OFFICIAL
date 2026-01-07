pub mod jobs;
pub mod models;
pub mod marketplace;
pub mod verification;
pub mod worker;
pub mod storage;
pub mod error;

pub use error::AiError;
pub use jobs::{Job, JobStatus, JobType};
pub use models::{Model, ModelRegistry};
pub use marketplace::Marketplace;
pub use verification::Verifier;
pub use worker::Worker;

pub type Result<T> = std::result::Result<T, AiError>;
