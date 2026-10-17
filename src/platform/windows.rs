use std::ffi::{CString, c_char, c_void};
use std::ptr;

const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const MEM_RELEASE: u32 = 0x8000;

const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE_READ: u32 = 0x20;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn VirtualAlloc(
        address: *mut c_void,
        size: usize,
        allocation_type: u32,
        protect: u32,
    ) -> *mut c_void;

    fn VirtualProtect(
        address: *mut c_void,
        size: usize,
        new_protect: u32,
        old_protect: *mut u32,
    ) -> i32;

    fn VirtualFree(address: *mut c_void, size: usize, free_type: u32) -> i32;
}

pub fn execute(bytes: &[u8], args: &[String]) -> Result<i32, String> {
    if bytes.is_empty() {
        return Err("Cannot execute empty machine code".to_string());
    }

    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| {
            CString::new(arg.as_bytes())
                .map_err(|_| "An argument contains an embedded NUL byte".to_string())
        })
        .collect::<Result<_, _>>()?;

    let mut argv: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    argv.push(ptr::null());

    unsafe {
        let memory = VirtualAlloc(
            ptr::null_mut(),
            bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if memory.is_null() {
            return Err(format!(
                "VirtualAlloc failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), bytes.len());

        let mut old_protect: u32 = 0;

        if VirtualProtect(memory, bytes.len(), PAGE_EXECUTE_READ, &mut old_protect) == 0 {
            let error = std::io::Error::last_os_error();
            VirtualFree(memory, 0, MEM_RELEASE);
            return Err(format!("VirtualProtect failed: {}", error));
        }

        // Fixed entry contract:
        //
        // extern "C" fn(argc: i32, argv: *const *const c_char) -> i32
        //
        // This is a custom entry convention, not a PE executable entry.
        let entry: unsafe extern "C" fn(i32, *const *const c_char) -> i32 =
            std::mem::transmute(memory);

        let argc = c_args.len() as i32;
        let status = entry(argc, argv.as_ptr());

        VirtualFree(memory, 0, MEM_RELEASE);

        Ok(status)
    }
}
