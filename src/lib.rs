//! # wun32job-rs
//!
//! A safe API for Windows' job objects, which can be used to set various limits to
//! processes associated with them.
//! See also https://docs.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-createjobobjectw
//!
//! # Using the higher level API
//!
//! The most basic API is getting and raw `JOBOBJECT_BASIC_LIMIT_INFORMATION`, modify it directly
//! and set it back to the job.
//!
//! It's important to remeber to set the needed `LimitFlags` for each limit used.
//!
//! ```edition2018
//! # use win32job::*;
//! # fn main() -> Result<(), JobError> {
//! use winapi::um::winnt::JOB_OBJECT_LIMIT_WORKINGSET;
//! let job = Job::create()?;
//! let mut info = job.query_extended_limit_info()?;
//! info.limit_working_memory(1 * 1024 * 1024,  4 * 1024 * 1024)
//!     .limit_priority_class(PriorityClass::BelowNormal);
//!
//! job.set_extended_limit_info(&mut info)?;
//! job.assign_current_process()?;
//! #   info.clear_limits();
//! #   job.set_extended_limit_info(&mut info)?;
//! #   Ok(())
//! # }
//! ```
//!
//! # Using the low level API
//!
//! The most basic API is getting and raw `JOBOBJECT_BASIC_LIMIT_INFORMATION`, modify it directly
//! and set it back to the job.
//!
//! It's important to remeber to set the needed `LimitFlags` for each limit used.
//!
//! ```edition2018
//! # use win32job::*;
//! # fn main() -> Result<(), JobError> {
//! use winapi::um::winnt::JOB_OBJECT_LIMIT_WORKINGSET;
//! let job = Job::create()?;
//! let mut info = job.query_extended_limit_info()?;
//!
//! info.0.BasicLimitInformation.MinimumWorkingSetSize = 1 * 1024 * 1024;
//! info.0.BasicLimitInformation.MaximumWorkingSetSize = 4 * 1024 * 1024;
//! info.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;
//!
//! job.set_extended_limit_info(&mut info)?;
//! job.assign_current_process()?;
//! #   info.clear_limits();
//! #   job.set_extended_limit_info(&mut info)?;
//! #   Ok(())
//! # }
//! ```
#[cfg(test)]
#[macro_use]
extern crate rusty_fork;
mod error;
mod job;
mod limits;
pub mod query;
pub mod utils;

pub use crate::error::JobError;
pub use crate::job::Job;
pub use crate::limits::{ExtendedLimitInfo, PriorityClass};
