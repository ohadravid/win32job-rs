use std::{io, mem, ptr};
use winapi::shared::minwindef::*;
use winapi::um::handleapi::*;
use winapi::um::jobapi2::*;
use winapi::um::winnt::*;
use crate::error::JobError;

pub use crate::utils::{get_current_process, get_process_memory_info};

#[derive(Debug)]
pub struct Job {
    handle: HANDLE,
}

impl Job {
    pub fn create() -> Result<Self, JobError> {
        let job_handle = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null()) };

        if job_handle.is_null() {
            return Err(JobError::CreateFailed(io::Error::last_os_error()));
        }

        Ok(Job { handle: job_handle })
    }

    pub fn basic_limit_info(&self) -> Result<JOBOBJECT_BASIC_LIMIT_INFORMATION, JobError> {
        let mut info: JOBOBJECT_BASIC_LIMIT_INFORMATION = unsafe { mem::zeroed() };
        let return_value = unsafe {
            QueryInformationJobObject(
                self.handle,
                JobObjectBasicLimitInformation,
                &mut info as *mut _ as LPVOID,
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

    pub fn set_basic_limit_info(
        &self,
        basic_info: &mut JOBOBJECT_BASIC_LIMIT_INFORMATION,
    ) -> Result<(), JobError> {
        let return_value = unsafe {
            SetInformationJobObject(
                self.handle,
                JobObjectBasicLimitInformation,
                basic_info as *mut _ as LPVOID,
                mem::size_of_val(basic_info) as DWORD,
            )
        };

        if return_value == 0 {
            Err(JobError::SetInfoFailed(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn set_extended_limit_info(
        &self,
        basic_info: &mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    ) -> Result<(), JobError> {
        let return_value = unsafe {
            SetInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                basic_info as *mut _ as LPVOID,
                mem::size_of_val(basic_info) as DWORD,
            )
        };

        if return_value == 0 {
            Err(JobError::SetInfoFailed(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn extended_limit_info(&self) -> Result<JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobError> {
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { mem::zeroed() };
        let return_value = unsafe {
            QueryInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                &mut info as *mut _ as LPVOID,
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

    pub fn assign_process(&self, proc_handle: HANDLE) -> Result<(), JobError> {
        let return_value = unsafe { AssignProcessToJobObject(self.handle, proc_handle) };

        if return_value == 0 {
            Err(JobError::AssignFailed(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

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
    use crate::utils::{get_current_process, get_process_memory_info};
    use crate::Job;
    use winapi::um::winnt::JOB_OBJECT_LIMIT_WORKINGSET;

    #[test]
    fn it_works() {
        let job = Job::create().unwrap();

        let mut info = job.basic_limit_info().unwrap();

        assert_eq!(info.LimitFlags, 0);

        // This is the default.
        assert_eq!(info.SchedulingClass, 5);

        info.MinimumWorkingSetSize = 1 * 1024 * 1024;
        info.MaximumWorkingSetSize = 4 * 1024 * 1024;

        info.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;

        job.set_basic_limit_info(&mut info).unwrap();

        let test_vec_size = 8 * 1024 * 1024;
        let mut big_vec: Vec<u8> = Vec::with_capacity(test_vec_size);
        big_vec.resize_with(test_vec_size, || 1);

        let memory_info = get_process_memory_info(get_current_process()).unwrap();

        assert!(memory_info.WorkingSetSize >= info.MaximumWorkingSetSize);

        job.assign_current_process().unwrap();

        let memory_info = get_process_memory_info(get_current_process()).unwrap();

        assert!(memory_info.WorkingSetSize <= info.MaximumWorkingSetSize);

        // Clear limits.
        info.LimitFlags = 0;
        job.set_basic_limit_info(&mut info).unwrap();
    }

    #[test]
    fn extended_info() {
        let job = Job::create().unwrap();

        let mut info = job.extended_limit_info().unwrap();

        assert_eq!(info.BasicLimitInformation.LimitFlags, 0);

        job.set_extended_limit_info(&mut info).unwrap();
    }
}