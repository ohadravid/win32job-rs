use std::{io, mem};
use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;
use winapi::um::winbase::GetProcessAffinityMask;
use winapi::um::winnt::*;
use winapi::um::{processthreadsapi, psapi};

/// Return a pseudo handle to the current process.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess) for this function.
pub fn get_current_process() -> HANDLE {
    unsafe { processthreadsapi::GetCurrentProcess() }
}

/// Retrieves information about the memory usage of the specified process.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getprocessmemoryinfo) for this function.
pub fn get_process_memory_info(
    process_handle: HANDLE,
) -> Result<PROCESS_MEMORY_COUNTERS, io::Error> {
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { mem::zeroed() };
    let return_value = unsafe {
        psapi::GetProcessMemoryInfo(
            process_handle,
            &mut counters as *mut _,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };

    if return_value == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(counters)
    }
}

/// Retrieves the process affinity mask for the specified process and the system affinity mask for the system.
/// See also [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprocessaffinitymask) for this function.
pub fn get_process_affinity_mask(process_handle: HANDLE) -> Result<(usize, usize), io::Error> {
    let mut process_affinity_mask = 0usize;
    let mut system_affinity_mask = 0usize;

    let return_value = unsafe {
        GetProcessAffinityMask(
            process_handle,
            &mut process_affinity_mask as *mut _,
            &mut system_affinity_mask as *mut _,
        )
    };

    if return_value == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok((process_affinity_mask, system_affinity_mask))
    }
}
