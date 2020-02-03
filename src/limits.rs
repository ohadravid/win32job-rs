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

impl ExtendedLimitInfo {
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
    pub fn limit_kill_on_job_close(&mut self) -> &mut Self {
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        self
    }

    /// Causes all processes associated with the job to use the same priority class.
    pub fn limit_priority_class(&mut self, priority_class: PriorityClass) -> &mut Self {
        self.0.BasicLimitInformation.PriorityClass = priority_class as u32;
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PRIORITY_CLASS;

        self
    }

    /// Causes all processes associated with the job to use the same processor affinity.
    pub fn limit_affinity(&mut self, affinity: usize) -> &mut Self {
        self.0.BasicLimitInformation.Affinity = affinity;
        self.0.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_AFFINITY;

        self
    }

    /// Clear all limits set for this job.
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
