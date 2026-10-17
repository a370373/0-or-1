#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
pub use unix::execute;

#[cfg(target_os = "windows")]
pub use windows::execute;

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "windows"
)))]
compile_error!("01 currently supports Linux, Android, macOS, and Windows targets");
