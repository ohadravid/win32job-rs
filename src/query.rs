use std::{io, mem};
use winapi::shared::basetsd::*;
use winapi::shared::minwindef::*;
use winapi::um::jobapi2::*;
use winapi::um::winnt::*;

use crate::{Job, JobError};

#[repr(C)]
struct ProcessIdList {
    header: JOBOBJECT_BASIC_PROCESS_ID_LIST,
    list: [ULONG_PTR; 1024],
}

impl Job {
    /// Process identifier list for a job object.
    /// If the job is nested, the process identifier list consists of all processes
    /// associated with the job and its child jobs.
    pub fn query_process_id_list(&self) -> Result<Vec<usize>, JobError> {
        // TODO: We will get an error if there are more than 1024 processes in the job.
        // This can be fixed by calling `QueryInformationJobObject` a second time,
        // with a bigger list with the correct size (as returned from the first call).
        let mut proc_id_list = ProcessIdList {
            header: unsafe { mem::zeroed() },
            list: [0usize; 1024],
        };

        let return_value = unsafe {
            QueryInformationJobObject(
                self.handle(),
                JobObjectBasicProcessIdList,
                &mut proc_id_list as *mut _ as LPVOID,
                mem::size_of_val(&proc_id_list) as DWORD,
                0 as *mut _,
            )
        };

        if return_value == 0 {
            return Err(JobError::GetInfoFailed(io::Error::last_os_error()));
        }

        let list = &proc_id_list.list[..proc_id_list.header.NumberOfProcessIdsInList as usize];

        Ok(list.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::Job;

    #[test]
    fn query_proc_id() {
        let job = Job::create().unwrap();

        let pids = job.query_process_id_list().unwrap();
        assert_eq!(pids, []);

        job.assign_current_process().unwrap();

        let pids = job.query_process_id_list().unwrap();
        assert_eq!(pids.len(), 1);
    }
}
