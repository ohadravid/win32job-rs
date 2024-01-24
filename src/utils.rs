use std::{io, mem};

use windows::Win32::{
    Foundation::HANDLE,
    System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        Threading::{GetCurrentProcess, GetProcessAffinityMask},
    },
};

/// Return a pseudo handle to the current process.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess) for this function.
pub fn get_current_process() -> HANDLE {
    unsafe { GetCurrentProcess() }
}

/// Retrieves information about the memory usage of the specified process.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getprocessmemoryinfo) for this function.
pub fn get_process_memory_info(
    process_handle: HANDLE,
) -> Result<PROCESS_MEMORY_COUNTERS, io::Error> {
    let mut counters: PROCESS_MEMORY_COUNTERS = PROCESS_MEMORY_COUNTERS::default();
    unsafe {
        GetProcessMemoryInfo(
            process_handle,
            &mut counters as *mut _,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    }
    .map_err(|e| e.into())
    .map(|_| counters)
}

/// Retrieves the process affinity mask for the specified process and the system affinity mask for the system.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprocessaffinitymask) for this function.
pub fn get_process_affinity_mask(process_handle: HANDLE) -> Result<(usize, usize), io::Error> {
    let mut process_affinity_mask = 0usize;
    let mut system_affinity_mask = 0usize;

    unsafe {
        GetProcessAffinityMask(
            process_handle,
            &mut process_affinity_mask as *mut _,
            &mut system_affinity_mask as *mut _,
        )
    }
    .map_err(|e| e.into())
    .map(|_| (process_affinity_mask, system_affinity_mask))
}
