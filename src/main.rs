mod parser;
mod platform;

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;
use std::process::{self, Command};

/// Accept only files whose extension is exactly `.01`.
fn validate_01_path(path: &OsStr) -> Result<(), String> {
    let extension = Path::new(path).extension().ok_or_else(|| {
        format!(
            "invalid file extension: {} (expected .01)",
            path.to_string_lossy()
        )
    })?;

    if extension != OsStr::new("01") {
        return Err(format!(
            "invalid file extension: {} (expected .01)",
            path.to_string_lossy()
        ));
    }

    Ok(())
}

fn print_help() {
    println!(
        "\
01 - Raw native machine-code loader

Usage:
  01 <file.01> [args...]
  01 --raw <file.01> [args...]
  01 --check <file.01>
  01 --help

Format:
  .01 files contain only ASCII 0 and 1, with optional whitespace.
  Every 8 bits form one byte, most-significant bit first.

Execution:
  Normal execution starts a child process.
  The child loads the raw bytes into executable memory and calls
  the entry point using the fixed C ABI:

    int entry(int argc, const char *const argv[])

  argv[0] is the .01 file path. Additional command-line arguments
  follow it.

Supported target families:
  Linux, Android, macOS, Windows

Important:
  Machine code must match the target CPU architecture and OS ABI.
  Raw machine code is not sandboxed and can crash or compromise the
  child process. It is not an ELF/PE/Mach-O loader."
    );
}

fn check_file(path: &OsString) -> Result<usize, String> {
    validate_01_path(path)?;

    let source = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.to_string_lossy(), e))?;

    let bytes = parser::parse_bits(&source)?;

    println!(
        "OK: {} -> {} machine-code byte(s)",
        path.to_string_lossy(),
        bytes.len()
    );

    Ok(bytes.len())
}

fn launch_child(path: OsString, args: Vec<OsString>) -> i32 {
    if let Err(error) = validate_01_path(&path) {
        eprintln!("01: {}", error);
        return 2;
    }

    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("01: cannot locate its own executable: {}", error);
            return 1;
        }
    };

    let result = Command::new(executable)
        .arg("--__child-raw")
        .arg(path)
        .args(args)
        .status();

    match result {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("01: cannot start child process: {}", error);
            1
        }
    }
}

fn run_child(path: OsString, args: Vec<OsString>) -> i32 {
    if let Err(error) = validate_01_path(&path) {
        eprintln!("01: {}", error);
        return 126;
    }

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("01: cannot read {}: {}", path.to_string_lossy(), error);
            return 126;
        }
    };

    let bytes = match parser::parse_bits(&source) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("01: invalid .01 file: {}", error);
            return 126;
        }
    };

    let mut program_args = Vec::with_capacity(args.len() + 1);

    // argv[0] follows the conventional program-name rule.
    program_args.push(path.to_string_lossy().into_owned());

    program_args.extend(args.iter().map(|arg| arg.to_string_lossy().into_owned()));

    match platform::execute(&bytes, &program_args) {
        Ok(status) => status,
        Err(error) => {
            eprintln!("01: execution failed: {}", error);
            126
        }
    }
}

fn main() {
    let mut args = env::args_os().skip(1);

    let Some(first) = args.next() else {
        print_help();
        process::exit(0);
    };

    match first.to_string_lossy().as_ref() {
        "--help" | "-h" => {
            print_help();
        }

        "--check" => {
            let Some(path) = args.next() else {
                eprintln!("Usage: 01 --check <file.01>");
                process::exit(2);
            };

            if args.next().is_some() {
                eprintln!("Usage: 01 --check <file.01>");
                process::exit(2);
            }

            if let Err(error) = check_file(&path) {
                eprintln!("01: {}", error);
                process::exit(2);
            }
        }

        "--raw" => {
            let Some(path) = args.next() else {
                eprintln!("Usage: 01 --raw <file.01> [args...]");
                process::exit(2);
            };

            process::exit(launch_child(path, args.collect()));
        }

        "--__child-raw" => {
            let Some(path) = args.next() else {
                eprintln!("01: internal child mode requires a file path");
                process::exit(2);
            };

            process::exit(run_child(path, args.collect()));
        }

        _ => {
            // Default: 01 <file.01> [args...]
            process::exit(launch_child(first, args.collect()));
        }
    }
}
