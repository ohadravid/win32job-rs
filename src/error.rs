use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JobError {
    #[error("Failed to create job")]
    CreateFailed(io::Error),
    #[error("Failed to assign job")]
    AssignFailed(io::Error),
    #[error("Failed to set info for job")]
    SetInfoFailed(io::Error),
    #[error("Failed to get info for job")]
    GetInfoFailed(io::Error),
}
