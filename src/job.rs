use crate::error::JobError;
use crate::limits::ExtendedLimitInfo;
use std::{io, mem, ptr};
use winapi::shared::minwindef::*;
use winapi::um::handleapi::*;
use winapi::um::jobapi2::*;
use winapi::um::winnt::*;

pub use crate::utils::{get_current_process, get_process_memory_info};

#[derive(Debug)]
pub struct Job {
    handle: HANDLE,
}

unsafe impl Send for Job {}
unsafe impl Sync for Job {}

impl Job {
    /// Create an anonymous job object.
    pub fn create() -> Result<Self, JobError> {
        let job_handle = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null()) };

        if job_handle.is_null() {
            return Err(JobError::CreateFailed(io::Error::last_os_error()));
        }

        Ok(Job { handle: job_handle })
    }

    /// Create an anonymous job object and sets it's limit according to `info`.
    /// Note: This method shouldn't change the provided `info`, but the internal Windows API
    /// require a mutable pointer, which means this function requires &mut as well.
    pub fn create_with_limit_info(info: &mut ExtendedLimitInfo) -> Result<Self, JobError> {
        let job = Self::create()?;
        job.set_extended_limit_info(info)?;

        Ok(job)
    }

    /// Return the underlying handle to the job.
    /// Note that this handle will be closed once the `Job` object is dropped.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    /// Return the underlying handle to the job, consuming the job.
    /// Note that the handle will NOT be closed, so it is the caller's responsibly to close it.
    pub fn into_handle(self) -> HANDLE {
        let job = mem::ManuallyDrop::new(self);

        job.handle
    }

    /// Return basic and extended limit information for a job object.
    /// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information).
    pub fn query_extended_limit_info(&self) -> Result<ExtendedLimitInfo, JobError> {
        let mut info = ExtendedLimitInfo::new();

        let return_value = unsafe {
            QueryInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                &mut info.0 as *mut _ as LPVOID,
                mem::size_of_val(&info) as DWORD,
                0 as *mut _,
            )
        };

        if return_value == 0 {
            Err(JobError::GetInfoFailed(io::Error::last_os_error()))
        } else {
            Ok(info)
        }
    }

    /// Set the basic and extended limit information for a job object.
    /// Note: This method shouldn't change the provided `info`, but the internal Windows API
    /// require a mutable pointer, which means this function requires &mut as well.
    pub fn set_extended_limit_info(&self, info: &mut ExtendedLimitInfo) -> Result<(), JobError> {
        let return_value = unsafe {
            SetInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                &mut info.0 as *mut _ as LPVOID,
                mem::size_of_val(&info.0) as DWORD,
            )
        };

        if return_value == 0 {
            Err(JobError::SetInfoFailed(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    /// Assigns a process to the job object.
    /// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject).
    pub fn assign_process(&self, proc_handle: HANDLE) -> Result<(), JobError> {
        let return_value = unsafe { AssignProcessToJobObject(self.handle, proc_handle) };

        if return_value == 0 {
            Err(JobError::AssignFailed(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    /// Assigns the current process to the job object.
    pub fn assign_current_process(&self) -> Result<(), JobError> {
        let current_proc_handle = get_current_process();

        self.assign_process(current_proc_handle)
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Job;
    use winapi::um::winnt::JOB_OBJECT_LIMIT_WORKINGSET;

    #[test]
    fn it_works() {
        let job = Job::create().unwrap();

        let mut info = job.query_extended_limit_info().unwrap();

        assert_eq!(info.0.BasicLimitInformation.LimitFlags, 0);

        // This is the default.
        assert_eq!(info.0.BasicLimitInformation.SchedulingClass, 5);

        info.0.BasicLimitInformation.MinimumWorkingSetSize = 1 * 1024 * 1024;
        info.0.BasicLimitInformation.MaximumWorkingSetSize = 4 * 1024 * 1024;

        info.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;

        job.set_extended_limit_info(&mut info).unwrap();

        // Clear limits.
        info.0.BasicLimitInformation.LimitFlags = 0;
        job.set_extended_limit_info(&mut info).unwrap();
    }
}
