use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub fn execute(bytes: &[u8], args: &[String]) -> Result<i32, String> {
    if bytes.is_empty() {
        return Err("Cannot execute empty machine code".to_string());
    }

    // Build argv strings before allocating executable memory.
    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| {
            CString::new(arg.as_bytes())
                .map_err(|_| "An argument contains an embedded NUL byte".to_string())
        })
        .collect::<Result<_, _>>()?;

    let mut argv: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    // argv must be terminated by a null pointer.
    argv.push(ptr::null());

    unsafe {
        let memory = libc::mmap(
            ptr::null_mut(),
            bytes.len(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        );

        if memory == libc::MAP_FAILED {
            return Err(format!("mmap failed: {}", std::io::Error::last_os_error()));
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), bytes.len());

        // Change the region from writable to executable/readable.
        // This avoids leaving the code region writable during execution.
        if libc::mprotect(memory, bytes.len(), libc::PROT_READ | libc::PROT_EXEC) != 0 {
            let error = std::io::Error::last_os_error();
            libc::munmap(memory, bytes.len());
            return Err(format!("mprotect failed: {}", error));
        }

        // Fixed entry contract:
        //
        // extern "C" fn(argc: i32, argv: *const *const c_char) -> i32
        //
        // argv[0] is the .01 file path.
        // argv[1..] are the arguments supplied by the user.
        //
        // The entry function may return an exit code, or terminate
        // this child process through an OS-provided mechanism.
        let entry: unsafe extern "C" fn(i32, *const *const c_char) -> i32 =
            std::mem::transmute(memory);

        let argc = (c_args.len() - 0) as i32;
        let status = entry(argc, argv.as_ptr());

        libc::munmap(memory, bytes.len());

        Ok(status)
    }
}
