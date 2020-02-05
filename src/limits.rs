use std::mem;
use winapi::um::winbase::{
    ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
    IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS,
};
use winapi::um::winnt::*;

pub struct ExtendedLimitInfo(pub JOBOBJECT_EXTENDED_LIMIT_INFORMATION);

#[repr(u32)]
pub enum PriorityClass {
    Normal = NORMAL_PRIORITY_CLASS,
    Idle = IDLE_PRIORITY_CLASS,
    High = HIGH_PRIORITY_CLASS,
    Realtime = REALTIME_PRIORITY_CLASS,
    BelowNormal = BELOW_NORMAL_PRIORITY_CLASS,
    AboveNormal = ABOVE_NORMAL_PRIORITY_CLASS,
}

impl Default for ExtendedLimitInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Contains basic and extended limit information for a job object, with helper methods for
/// easy limit manipulation. To apply limits, pass the instance of this struct to
/// `Job::create_with_limit_info` or `job.set_extended_limit_info`.
impl ExtendedLimitInfo {
    /// Return an empty extended info objects, without any limits.
    pub fn new() -> Self {
        let inner: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { mem::zeroed() };
        ExtendedLimitInfo(inner)
    }

    /// Causes all processes associated with the job
    /// to use the same minimum and maximum working set sizes
    pub fn limit_working_memory(&mut self, min: usize, max: usize) -> &mut Self {
        self.0.BasicLimitInformation.MinimumWorkingSetSize = min;
        self.0.BasicLimitInformation.MaximumWorkingSetSize = max;

        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_WORKINGSET;

        self
    }

    /// Causes all processes associated with the job to terminate
    /// when the last handle to the job is closed.
    /// Note, that that `drop`ing the `Job` struct closes this handle, and if it's the only handle
    /// to the job **the current process will terminate** if it's assign to that job.
    pub fn limit_kill_on_job_close(&mut self) -> &mut Self {
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        self
    }

    /// Causes all processes associated with the job to use the same priority class.
    /// Note: Processes and threads cannot modify their priority class.
    /// The calling process must enable the `SE_INC_BASE_PRIORITY_NAME` privilege.
    pub fn limit_priority_class(&mut self, priority_class: PriorityClass) -> &mut Self {
        self.0.BasicLimitInformation.PriorityClass = priority_class as u32;
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PRIORITY_CLASS;

        self
    }

    /// Causes all processes in the job to use the same scheduling class.
    /// The valid values are 0 to 9.
    /// Use 0 for the least favorable scheduling class relative to other threads,
    /// and 9 for the most favorable scheduling class relative to other threads.
    /// By default, this value is 5.
    /// Note: To use a scheduling class greater than 5,
    /// the calling process must enable the `SE_INC_BASE_PRIORITY_NAME` privilege.
    pub fn limit_scheduling_class(&mut self, scheduling_class: u8) -> &mut Self {
        self.0.BasicLimitInformation.SchedulingClass = scheduling_class as u32;
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_SCHEDULING_CLASS;

        self
    }

    /// Causes all processes associated with the job to use the same processor affinity.
    pub fn limit_affinity(&mut self, affinity: usize) -> &mut Self {
        self.0.BasicLimitInformation.Affinity = affinity;
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_AFFINITY;

        self
    }

    /// Clear all limits.
    pub fn clear_limits(&mut self) -> &mut Self {
        self.0.BasicLimitInformation.LimitFlags = 0;

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_current_process, get_process_affinity_mask, get_process_memory_info};
    use crate::{Job, PriorityClass};
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn working_mem_limits() {
            let job = Job::create().unwrap();
            let mut info = job.query_extended_limit_info().unwrap();

            let min = 1 * 1024 * 1024;
            let max = 4 * 1024 * 1024;
            info.limit_working_memory(min, max);

            job.set_extended_limit_info(&mut info).unwrap();

            let test_vec_size = 8 * 1024 * 1024;
            let mut big_vec: Vec<u8> = Vec::with_capacity(test_vec_size);
            big_vec.resize_with(test_vec_size, || 1);

            let memory_info = get_process_memory_info(get_current_process()).unwrap();
            println!("{}", memory_info.WorkingSetSize);
            assert!(memory_info.WorkingSetSize >= max);

            job.assign_current_process().unwrap();

            let memory_info = get_process_memory_info(get_current_process()).unwrap();

            assert!(memory_info.WorkingSetSize <= max);

            info.clear_limits();

            job.set_extended_limit_info(&mut info).unwrap();
        }
    }

    rusty_fork_test! {
        #[test]
        fn kill_on_job_close_limits() {
            let job = Job::create().unwrap();
            let mut info = job.query_extended_limit_info().unwrap();

            info.limit_kill_on_job_close();

            job.set_extended_limit_info(&mut info).unwrap();

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

            let mut info = job.query_extended_limit_info().unwrap();

            info.limit_priority_class(PriorityClass::BelowNormal);

            job.set_extended_limit_info(&mut info).unwrap();

            let info = job.query_extended_limit_info().unwrap();

            assert_eq!(info.0.BasicLimitInformation.PriorityClass, PriorityClass::BelowNormal as u32);
        }
    }

    rusty_fork_test! {
        #[test]
        fn scheduling_class_limits() {
            let job = Job::create().unwrap();

            let mut info = job.query_extended_limit_info().unwrap();

            info.limit_scheduling_class(1);

            job.set_extended_limit_info(&mut info).unwrap();

            let info = job.query_extended_limit_info().unwrap();

            assert_eq!(info.0.BasicLimitInformation.SchedulingClass, 1);
        }
    }

    rusty_fork_test! {
        #[test]
        fn affinity_limits() {
            let job = Job::create().unwrap();

            let mut info = job.query_extended_limit_info().unwrap();

            info.limit_affinity(1);

            job.set_extended_limit_info(&mut info).unwrap();

            let (proc_affinity, _) = get_process_affinity_mask(get_current_process()).unwrap();
            assert_ne!(proc_affinity, 1);

            job.assign_current_process().unwrap();

            let (proc_affinity, _) = get_process_affinity_mask(get_current_process()).unwrap();
            assert_eq!(proc_affinity, 1);
        }
    }
}
