// Don't use console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env::args,
    io::Error,
    mem::size_of,
    path::Path,
    ptr::{null, null_mut},
};
use wedge_lib::{
    install::{uninstall, UNINSTALLER_NAME},
    win32::*,
    *,
};
use winapi::{
    shared::minwindef::FALSE,
    um::{
        processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW},
        winbase::{CopyFileW, MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT},
        winuser::{
            MessageBoxExW, IDOK, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_TOPMOST,
        },
    },
};

/// Entry
#[cfg(windows)]
fn main() {
    // Run uninstall and notify user of any error
    if let Err(e) = uninstall_proc() {
        unsafe {
            MessageBoxExW(
                null_mut(),
                TEXT!(&e.to_string()),
                TEXT!("Error Uninstalling EdgeDeflector"),
                MB_ICONERROR | MB_OK | MB_TOPMOST,
                0,
            )
        };
    }
}

/// Runs the uninstallation procedure
fn uninstall_proc() -> Result<(), Error> {
    // If program is executed with "run-uninstall" command line arg that means it
    // was already copied to a temp location where it can safely delete the original
    // files. Proceed with uninstallation. Ignore as many errors as possible.
    if args().any(|arg| arg == "run-uninstall") {
        uninstall()?;
    }
    // Ask user if they want to proceed with uninstallation and if they do copy the uninstaller
    // executable to a temp location and execute with "run-uninstall" command line arg
    // Code derived from https://stackoverflow.com/questions/10319526/understanding-a-self-deleting-program-in-c
    else {
        unsafe {
            // Ask user if they want to proceed with uninstallation before proceeding
            if MessageBoxExW(
                null_mut(),
                TEXT!(
                    "This will uninstall EdgeDeflector and restore system defaults. Proceed with \
                     uninstallation?"
                ),
                TEXT!("Uninstall EdgeDeflector"),
                MB_ICONWARNING | MB_OKCANCEL | MB_TOPMOST,
                0,
            ) == IDOK
            {
                let source = get_self_location()?;
                let target = Path::new(&get_temp_location()?).join(UNINSTALLER_NAME);
                let target = target.to_str().unwrap();

                // Make a copy of this uninstaller in the %temp% file overwriting any copies
                if CopyFileW(TEXT!(&source), TEXT!(&target), FALSE) == 0 {
                    return Err(Error::last_os_error());
                }

                // Movefile to empty so the uninstaller will eventually be deleted
                MoveFileExW(TEXT!(&target), null(), MOVEFILE_DELAY_UNTIL_REBOOT);

                let mut pi = PROCESS_INFORMATION {
                    hProcess: null_mut(),
                    hThread: null_mut(),
                    dwProcessId: 0 as _,
                    dwThreadId: 0 as _,
                };
                let mut si = STARTUPINFOW {
                    cb: size_of::<STARTUPINFOW>() as _,
                    lpReserved: null_mut(),
                    lpDesktop: null_mut(),
                    lpTitle: null_mut(),
                    dwX: 0 as _,
                    dwY: 0 as _,
                    dwXSize: 0 as _,
                    dwYSize: 0 as _,
                    dwXCountChars: 0 as _,
                    dwYCountChars: 0 as _,
                    dwFillAttribute: 0 as _,
                    dwFlags: 0 as _,
                    wShowWindow: 0 as _,
                    cbReserved2: 0 as _,
                    lpReserved2: null_mut(),
                    hStdInput: null_mut(),
                    hStdOutput: null_mut(),
                    hStdError: null_mut(),
                };

                // Invoke the temp uninstaller with command line args "run-uninstall"
                if CreateProcessW(
                    TEXT!(&target),
                    TEXT!("run-uninstall"),
                    null_mut(),
                    null_mut(),
                    FALSE,
                    0,
                    null_mut(),
                    null(),
                    &mut si,
                    &mut pi,
                ) == 0
                {
                    return Err(Error::last_os_error());
                }
            }
        }
    }
    Ok(())
}
