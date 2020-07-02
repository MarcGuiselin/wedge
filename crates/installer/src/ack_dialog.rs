use std::{io::Error, ptr::null_mut};
use wedge_lib::{win32::*, *};
use winapi::{
    shared::{
        minwindef::{FALSE, LPARAM, LRESULT, UINT, WPARAM},
        windef::{HWND, RECT},
    },
    um::{
        commctrl::{NM_CLICK, NM_RETURN, PNMLINK},
        libloaderapi::GetModuleHandleW,
        shellapi::ShellExecuteW,
        winnt::LPWSTR,
        winuser::{
            BeginPaint, DialogBoxParamW, DrawIconEx, EndDialog, EndPaint, GetDlgItem, LoadImageW,
            SendMessageW, ICON_BIG, ICON_SMALL, IDCANCEL, IDOK, IMAGE_ICON, LPNMHDR, PAINTSTRUCT,
            SW_SHOWNORMAL, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_INITDIALOG, WM_NOTIFY, WM_PAINT,
            WM_SETICON, WM_SETTEXT,
        },
    },
};

// Settings
const USER_ACKNOWLEDGEMENT: &str = concat!(
    r#"Wedge Open-Source Installer v"#,
    env!("CARGO_PKG_VERSION"),
    "\n",
    r#"<A HREF="https://github.com/MarcGuiselin/wedge/">Source (github)</A> | <A HREF="https://github.com/MarcGuiselin/wedge/blob/master/LICENSE">GPL v3.0</A>"#,
    "\n\nThis will override your system's defaults to use your default browser for microsoft-edge \
     links. System defaults can be restored by uninstalling Wedge in Windows 'Apps and \
     Features.'\n\nAcknowledge and continue?"
);

// Resource Ids
const ICON_RESOURCE: LPWSTR = MAKEINTRESOURCE!(1);
const ACK_DIALOG: LPWSTR = MAKEINTRESOURCE!(100);

// Dialog common control ids
const ID_SYSLINK: i32 = 1000;

/// Temporarily stores user response (Not very rusty I know but handled safely)
static mut USER_ACK_RESPONSE: bool = false;

/// Displays user acknowledgement dialog and returns user response
pub fn display() -> Result<bool, Error> {
    unsafe {
        // Load common control classes
        init_common_controls()?;

        // Safe reset of user response
        USER_ACK_RESPONSE = false;

        // Open dialog and output errors
        if DialogBoxParamW(
            null_mut(),
            ACK_DIALOG,
            null_mut(),
            Some(dialog_proc),
            0 as _,
        ) < 0
        {
            Err(Error::last_os_error())
        } else {
            // Safely return user response
            Ok(USER_ACK_RESPONSE)
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
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Setup dialog window after initiation
        WM_INITDIALOG => {
            let hinstance = GetModuleHandleW(null_mut());

            // Load pixel perfect menu icon
            let hicon = LoadImageW(hinstance, ICON_RESOURCE, IMAGE_ICON, 16, 16, 0);
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as _, hicon as _);

            // Load taskbar icon
            let hicon = LoadImageW(hinstance, ICON_RESOURCE, IMAGE_ICON, 48, 48, 0);
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as _, hicon as _);

            // Set user acknowledgment text
            let sys_link = GetDlgItem(hwnd, ID_SYSLINK);
            SendMessageW(sys_link, WM_SETTEXT, 0, TEXT!(USER_ACKNOWLEDGEMENT) as _);

            true as _
        }

        // Draw large icon
        WM_PAINT => {
            let mut ps = PAINTSTRUCT {
                hdc: 0 as _,
                fErase: FALSE,
                rcPaint: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                fRestore: FALSE,
                fIncUpdate: FALSE,
                rgbReserved: [0; 32],
            };

            let hdc = BeginPaint(hwnd, &mut ps);
            let hinstance = GetModuleHandleW(null_mut());
            let himg = LoadImageW(hinstance, ICON_RESOURCE, IMAGE_ICON, 128, 128, 0 as _);
            DrawIconEx(hdc, 2, 4, himg as _, 128, 128, 0, null_mut(), 0x0003);
            EndPaint(hwnd, &ps);
            true as _
        }

        // Proccess button commands
        WM_COMMAND => match wparam as i32 {
            IDOK => {
                USER_ACK_RESPONSE = true;
                EndDialog(hwnd, 0);
                true as _
            }
            IDCANCEL => {
                USER_ACK_RESPONSE = false;
                EndDialog(hwnd, 0);
                true as _
            }
            _ => false as _,
        },

        // Open clicked link
        // https://docs.microsoft.com/en-us/windows/win32/controls/use-syslihnk-notifications
        WM_NOTIFY => {
            let code = (*(lparam as LPNMHDR)).code;
            if code == NM_CLICK || code == NM_RETURN {
                let item = (*(lparam as PNMLINK)).item;
                ShellExecuteW(
                    null_mut(),
                    TEXT!("open"),
                    item.szUrl.as_ptr(),
                    null_mut(),
                    null_mut(),
                    SW_SHOWNORMAL,
                );
            }
            true as _
        }

        WM_CLOSE | WM_DESTROY => {
            EndDialog(hwnd, 0);
            false as _
        }

        _ => false as _,
    }
}
