use std::{io::Error, ptr::null_mut};
use wedge_lib::{
    install::{install, STEP_COUNT, STEP_INTERVAL},
    win32::*,
    *,
};
use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        commctrl::{PBM_SETRANGE, PBM_SETSTATE, PBM_SETSTEP, PBM_STEPIT, PBST_ERROR},
        libloaderapi::GetModuleHandleW,
        winnt::LPWSTR,
        winuser::{
            DialogBoxParamW, EndDialog, GetDlgItem, HideCaret, KillTimer, LoadImageW, SendMessageW,
            SetTimer, EM_REPLACESEL, EM_SCROLL, EM_SETSEL, ICON_BIG, ICON_SMALL, IMAGE_ICON,
            SB_PAGEDOWN, WM_CLOSE, WM_DESTROY, WM_INITDIALOG, WM_SETCURSOR, WM_SETICON, WM_TIMER,
        },
    },
};

// Resource Ids
const ICON_RESOURCE: LPWSTR = MAKEINTRESOURCE!(1);
const INSTALL_DIALOG: LPWSTR = MAKEINTRESOURCE!(101);

// Dialog common control ids
const ID_PROGRESS: i32 = 1001;
const ID_RICH_EDIT: i32 = 1002;

/// Displays install progress dialog and returns success
pub fn display() -> Result<(), Error> {
    unsafe {
        // Load common control classes
        init_common_controls()?;

        // Open dialog and output errors
        if DialogBoxParamW(
            null_mut(),
            INSTALL_DIALOG,
            null_mut(),
            Some(dialog_proc),
            0 as _,
        ) < 0
        {
            Err(Error::last_os_error())
        } else {
            // Safely return user response
            Ok(())
        }
    }
}

/// Dialog callback
///
/// https://docs.microsoft.com/en-us/previous-versions/ms960202(v%3Dmsdn.10)
unsafe extern "system" fn dialog_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_SETCURSOR => {
            let htext = GetDlgItem(hwnd, ID_RICH_EDIT);
            HideCaret(htext);
            false as _
        }

        // Setup dialog window after initiation
        WM_INITDIALOG => {
            let hinstance = GetModuleHandleW(null_mut());

            // Load pixel perfect menu icon
            let hicon = LoadImageW(hinstance, ICON_RESOURCE, IMAGE_ICON, 16, 16, 0);
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as _, hicon as _);

            // Load taskbar icon
            let hicon = LoadImageW(hinstance, ICON_RESOURCE, IMAGE_ICON, 48, 48, 0);
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as _, hicon as _);

            // Hide caret
            let htext = GetDlgItem(hwnd, ID_RICH_EDIT);
            HideCaret(htext);

            // Get our progress bar
            let hprogress = GetDlgItem(hwnd, ID_PROGRESS);

            // Set the range and increment of the progress bar
            // Use increment of 10 to allow smooth animation
            SendMessageW(hprogress, PBM_SETRANGE, 0, STEP_COUNT << 16);
            SendMessageW(hprogress, PBM_SETSTEP, 1, 0);

            // First step
            SendMessageW(hprogress, PBM_STEPIT, 0, 0);

            // Start installer steps
            SetTimer(hwnd, 1, STEP_INTERVAL, None);
            true as _
        }

        WM_TIMER => {
            // Get our progress bar
            let hprogress = GetDlgItem(hwnd, ID_PROGRESS);

            // Kill timer otherwise we'll start repeating steps
            KillTimer(hwnd, wparam);

            // Run installer step and log error or success message
            let mut log = match install(wparam) {
                Ok(msg) => {
                    // Start timer to execute next step, as long as there is a next step
                    if wparam + 1 < STEP_COUNT as usize {
                        SetTimer(hwnd, wparam + 1, STEP_INTERVAL, None);
                    }

                    // Move progress bar forward a step
                    SendMessageW(hprogress, PBM_STEPIT, 0, 0);

                    // Log success message to user
                    format!(
                        "Step {}/{}\r\n\t{}",
                        wparam,
                        STEP_COUNT - 1,
                        msg.replace("\n", "\r\n\t")
                    )
                }
                Err(e) => {
                    // Set progress bar to error state
                    SendMessageW(hprogress, PBM_SETSTATE, PBST_ERROR as _, 0);

                    // Log error
                    format!("Error on step {}/{} {}", wparam, STEP_COUNT - 1, e)
                }
            };
            log.push_str("\r\n");

            // Log message to user
            let htext = GetDlgItem(hwnd, ID_RICH_EDIT);
            SendMessageW(htext, EM_SETSEL as _, 0, isize::max_value());
            SendMessageW(
                htext,
                EM_SETSEL as _,
                usize::max_value(),
                isize::max_value(),
            );
            SendMessageW(htext, EM_REPLACESEL as _, 0, TEXT!(&log) as _);
            SendMessageW(htext, EM_SCROLL as _, SB_PAGEDOWN as _, 0);
            HideCaret(htext);
            true as _
        }

        WM_CLOSE | WM_DESTROY => {
            EndDialog(hwnd, 0);
            false as _
        }

        _ => false as _,
    }
}
