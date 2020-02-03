use std::mem;
use winapi::um::winnt::{JOB_OBJECT_LIMIT_WORKINGSET, JOBOBJECT_BASIC_LIMIT_INFORMATION,
                        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PRIORITY_CLASS,
                        JOB_OBJECT_LIMIT_AFFINITY};
use winapi::um::winbase::{NORMAL_PRIORITY_CLASS,
                          IDLE_PRIORITY_CLASS,
                          HIGH_PRIORITY_CLASS,
                          REALTIME_PRIORITY_CLASS,
                          BELOW_NORMAL_PRIORITY_CLASS,
                          ABOVE_NORMAL_PRIORITY_CLASS};

use crate::{Job, JobError};

#[repr(u32)]
pub enum PriorityClass {
    Normal = NORMAL_PRIORITY_CLASS,
    Idle = IDLE_PRIORITY_CLASS,
    High = HIGH_PRIORITY_CLASS,
    Realtime = REALTIME_PRIORITY_CLASS,
    BelowNormal = BELOW_NORMAL_PRIORITY_CLASS,
    AboveNormal = ABOVE_NORMAL_PRIORITY_CLASS,
}

impl Job {
    /// Causes all processes associated with the job
    /// to use the same minimum and maximum working set sizes
    pub fn limit_working_memory(&self, min: usize, max: usize) -> Result<(), JobError> {
        let mut info = self.basic_limit_info()?;

        info.MinimumWorkingSetSize = min;
        info.MaximumWorkingSetSize = max;

        info.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;

        self.set_basic_limit_info(&mut info)
    }

    /// Causes all processes associated with the job to terminate
    /// when the last handle to the job is closed.
    pub fn limit_kill_on_job_close(&self) -> Result<(), JobError> {
        let mut info = self.extended_limit_info()?;
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        self.set_extended_limit_info(&mut info)
    }

    /// Causes all processes associated with the job to use the same priority class.
    pub fn limit_priority_class(&self, priority_class: PriorityClass) -> Result<(), JobError> {
        let mut info = self.basic_limit_info()?;

        info.PriorityClass = priority_class as u32;
        info.LimitFlags |= JOB_OBJECT_LIMIT_PRIORITY_CLASS;

        self.set_basic_limit_info(&mut info)
    }

    /// Causes all processes associated with the job to use the same processor affinity.
    pub fn limit_affinity(&self, affinity: usize) -> Result<(), JobError> {
        let mut info = self.basic_limit_info()?;

        info.Affinity = affinity;
        info.LimitFlags |= JOB_OBJECT_LIMIT_AFFINITY;

        self.set_basic_limit_info(&mut info)
    }

    /// Clear all limits set for this job.
    pub fn clear_limits(&self) -> Result<(), JobError> {
        let mut info: JOBOBJECT_BASIC_LIMIT_INFORMATION = unsafe { mem::zeroed() };

        // Clear limits explicitly.
        info.LimitFlags = 0;

        self.set_basic_limit_info(&mut info)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_current_process, get_process_memory_info, get_process_affinity_mask};
    use crate::{Job, PriorityClass};
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

    rusty_fork_test! {
        #[test]
        fn kill_on_job_close_limits() {
            let job = Job::create().unwrap();
            job.limit_kill_on_job_close().unwrap();

            job.assign_current_process().unwrap();

            drop(job);

            // Never reached.
            panic!();
        }
    }

    rusty_fork_test! {
        #[test]
        fn priority_class_limits() {
            let job = Job::create().unwrap();
            job.limit_priority_class(PriorityClass::BelowNormal).unwrap();

            let info = job.basic_limit_info().unwrap();

            assert_eq!(info.PriorityClass, PriorityClass::BelowNormal as u32);
        }
    }

    rusty_fork_test! {
        #[test]
        fn affinity_limits() {
            let job = Job::create().unwrap();
            job.limit_affinity(1).unwrap();


            let (proc_affinity, _) = get_process_affinity_mask(get_current_process()).unwrap();
            assert_ne!(proc_affinity, 1);

            job.assign_current_process().unwrap();

            let (proc_affinity, _) = get_process_affinity_mask(get_current_process()).unwrap();
            assert_eq!(proc_affinity, 1);
        }
    }
}
