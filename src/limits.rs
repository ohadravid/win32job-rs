use std::mem;
use winapi::um::winnt::{JOB_OBJECT_LIMIT_WORKINGSET, JOBOBJECT_BASIC_LIMIT_INFORMATION};
use crate::{Job, JobError};

impl Job {
    pub fn limit_working_memory(&self, min: usize, max: usize) -> Result<(), JobError> {
        let mut info = self.basic_limit_info()?;

        info.MinimumWorkingSetSize = min;
        info.MaximumWorkingSetSize = max;

        info.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;

        self.set_basic_limit_info(&mut info)
    }

    pub fn clear_limits(&self) -> Result<(), JobError> {
        let mut info: JOBOBJECT_BASIC_LIMIT_INFORMATION = unsafe { mem::zeroed() };

        // Clear limits explicitly.
        info.LimitFlags = 0;

        self.set_basic_limit_info(&mut info)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_current_process, get_process_memory_info};
    use crate::Job;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn working_mem_limits() {
            let job = Job::create().unwrap();
            let min = 1 * 1024 * 1024;
            let max = 4 * 1024 * 1024;
            job.limit_working_memory(min, max).unwrap();

            let test_vec_size = 8 * 1024 * 1024;
            let mut big_vec: Vec<u8> = Vec::with_capacity(test_vec_size);
            big_vec.resize_with(test_vec_size, || 1);

            let memory_info = get_process_memory_info(get_current_process()).unwrap();
            println!("{}", memory_info.WorkingSetSize);
            assert!(memory_info.WorkingSetSize >= max);

            job.assign_current_process().unwrap();

            let memory_info = get_process_memory_info(get_current_process()).unwrap();

            assert!(memory_info.WorkingSetSize <= max);

            job.clear_limits().unwrap();
        }
    }
}
