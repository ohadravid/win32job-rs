//! # wun32job-rs
//!
//! A safe API for Windows' job objects, which can be used to set various limits to
//! processes associated with them.
//! See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-createjobobjectw).
//!
//! # Using the higher level API
//!
//! After getting an `ExtendedLimitInfo` object, either by querying the info of a job
//! or by creating an empty one using `new()`, use helper methods to configure
//! the required limits, and finally set the info to the job.
//!
//! ```edition2018
//! use win32job::*;
//! # fn main() -> Result<(), JobError> {
//!
//! let mut info = ExtendedLimitInfo::new();
//!
//! info.limit_working_memory(1 * 1024 * 1024,  4 * 1024 * 1024)
//!     .limit_priority_class(PriorityClass::BelowNormal);
//!
//! let job = Job::create_with_limit_info(&mut info)?;
//! job.assign_current_process()?;
//! #   info.clear_limits();
//! #   job.set_extended_limit_info(&mut info)?;
//! #   Ok(())
//! # }
//! ```
//!
//! Which is equivalnent to:
//! ```edition2018
//! use win32job::*;
//! # fn main() -> Result<(), JobError> {
//!
//! let job = Job::create()?;
//! let mut info = job.query_extended_limit_info()?;
//!
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
//! The most basic API is getting an `ExtendedLimitInfo` object and
//! manipulating the raw `JOBOBJECT_BASIC_LIMIT_INFORMATION`, and then set it back to the job.
//!
//! It's important to remeber to set the needed `LimitFlags` for each limit used.
//!
//! ```edition2018
//! use win32job::*;
//! # fn main() -> Result<(), JobError> {
//! use winapi::um::winnt::JOB_OBJECT_LIMIT_WORKINGSET;
//!
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
mod query;
pub mod utils;

pub use crate::error::JobError;
pub use crate::job::Job;
pub use crate::limits::{ExtendedLimitInfo, PriorityClass};

// Cannot use `cfg(test)` here since `rustdoc` won't look at it.
#[cfg(debug_assertions)]
mod test_readme {
    macro_rules! calculated_doc {
        ($doc:expr, $id:ident) => {
            #[doc = $doc]
            enum $id {}
        }
    }

    calculated_doc!(include_str!("../README.md"), _DoctestReadme);
}
