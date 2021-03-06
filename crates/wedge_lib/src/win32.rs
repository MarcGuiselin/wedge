#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use super::com::ComPtr;
use std::{
    ffi::OsStr,
    io::{Error, ErrorKind},
    iter::once,
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
};
use widestring::U16CString;
use winapi::{
    shared::{
        minwindef::{MAX_PATH, TRUE},
        ntdef::NULL,
        winerror::{SUCCEEDED, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoInitializeEx},
        fileapi::GetTempPathW,
        libloaderapi::{GetModuleFileNameW, GetModuleHandleW},
        objbase::COINIT_MULTITHREADED,
        shellapi::ShellExecuteW,
        shlobj::{SHGetFolderPathW, CSIDL_LOCAL_APPDATA, CSIDL_PROGRAMS},
        winuser::SW_SHOWNORMAL,
    },
};

/// Winapi's MAKEINTRESOURCE macro made available
#[macro_export]
macro_rules! MAKEINTRESOURCE {
    ($i:expr) => {
        $i as u16 as usize as winapi::um::winnt::LPWSTR
    };
}

/// Rust implementation of Winuser.h MAKELPARAM macro
#[macro_export]
macro_rules! MAKELPARAM {
    ($b:expr, $a:expr) => {
        (($a << 16) + $b) as isize
    };
}

/// Kinda like microsoft's text macro except itW returns pointer to wide encoded string
#[macro_export]
macro_rules! TEXT {
    ($i:expr) => {
        win32_string($i).as_mut_ptr()
    };
}

/// We have to encode text to wide format for Windows
pub fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

/// Executes a shell open command
pub fn shell_execute(command: &str) {
    unsafe {
        ShellExecuteW(
            null_mut(),
            TEXT!("open"),
            TEXT!(command),
            null_mut(),
            null_mut(),
            SW_SHOWNORMAL,
        );
    }
}

/// Loads common control classes
pub fn init_common_controls() -> Result<(), Error> {
    // Disabled as I don't believe this makes a difference for dialogs

    // let lp_init_ctrls = INITCOMMONCONTROLSEX {
    //     dwSize: size_of::<INITCOMMONCONTROLSEX>() as _,
    //     dwICC:  ICC_PROGRESS_CLASS | ICC_LINK_CLASS | ICC_STANDARD_CLASSES,
    // };
    // if unsafe { InitCommonControlsEx(&lp_init_ctrls) } == 0 {
    //     Err(Error::last_os_error())
    // } else {
    //     Ok(())
    // }

    Ok(())
}

/// Returns location of this running executable
#[cfg(windows)]
pub fn get_self_location() -> Result<String, Error> {
    let mut pf: [u16; MAX_PATH] = [0; MAX_PATH];
    let buffer = pf.as_mut_ptr();
    unsafe {
        // Get the running handle first
        let hinstance = GetModuleHandleW(null_mut());

        // 0 is error
        if GetModuleFileNameW(hinstance, buffer, MAX_PATH as _) == 0 {
            Err(Error::new(
                ErrorKind::Other,
                "Could not get default install location!",
            ))
        } else {
            Ok(U16CString::from_ptr_str(buffer).to_string_lossy())
        }
    }
}

/// Returns location of %temp% folder
/// "C:\Users\user\AppData\Local\Temp"
#[cfg(windows)]
pub fn get_temp_location() -> Result<String, Error> {
    let mut pf: [u16; MAX_PATH] = [0; MAX_PATH];
    let buffer = pf.as_mut_ptr();
    unsafe {
        // 0 is error
        if GetTempPathW(MAX_PATH as _, buffer) == 0 {
            Err(Error::new(ErrorKind::Other, "Could not get temp location!"))
        } else {
            Ok(U16CString::from_ptr_str(buffer).to_string_lossy())
        }
    }
}

fn shell_get_folder_path(id: i32) -> Result<String, Error> {
    let mut pf: [u16; MAX_PATH] = [0; MAX_PATH];
    let buffer = pf.as_mut_ptr();
    unsafe {
        // S_OK is success
        if SHGetFolderPathW(null_mut(), id, null_mut(), 0, buffer) == S_OK {
            Ok(U16CString::from_ptr_str(buffer).to_string_lossy())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Could not get default install location!",
            ))
        }
    }
}

/// Returns location of local appdata folder
/// "C:\Users\user\AppData\Local\"
#[cfg(windows)]
pub fn get_local_install_location() -> Result<String, Error> {
    shell_get_folder_path(CSIDL_LOCAL_APPDATA)
}

/// Returns location of user's program shortcuts
/// "C:\Users\user\AppData\Roaming\Microsoft\Windows\Start Menu\Programs"
#[cfg(windows)]
pub fn get_user_start_menu_location() -> Result<String, Error> {
    shell_get_folder_path(CSIDL_PROGRAMS)
}

/// Creates a link to another file
/// https://docs.microsoft.com/en-us/windows/win32/shell/links
#[cfg(windows)]
pub fn create_link(path: &str, target: &str, desc: &str) -> Result<(), Error> {
    use com::*;
    let mut hres;

    unsafe {
        // Initialize COM
        hres = CoInitializeEx(0 as _, COINIT_MULTITHREADED);
        if SUCCEEDED(hres) {
            // Get a pointer to the IShellLink interface.
            let mut psl = NULL;
            hres = CoCreateInstance(
                &CLSID_ShellLink,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IShellLinkW,
                &mut psl,
            );
            if SUCCEEDED(hres) {
                // Create smart rust COM pointer
                let psl = ComPtr::from_raw(psl as *mut IShellLinkW);

                // Set the path to the shortcut target and add the description.
                psl.SetPath(TEXT!(target));
                psl.SetDescription(TEXT!(desc));
                // psl.SetIconLocation(TEXT!(target), 0);

                // Query IShellLink for the IPersistFile interface, used for saving the
                // shortcut in persistent storage.
                let mut ppf = NULL;
                hres = psl.QueryInterface(&IID_IPersistFile, &mut ppf);
                if SUCCEEDED(hres) {
                    // Create smart rust COM pointer
                    let ppf = ComPtr::from_raw(ppf as *mut IPersistFile);

                    // Save the link by calling IPersistFile::Save.
                    hres = ppf.Save(TEXT!(path), TRUE);
                }
            }
        }
    }

    // Return error if any stage of link creation failed
    if SUCCEEDED(hres) {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "Error creating link"))
    }
}

/// Stupid, ugly com bindings >:(
mod com {
    use winapi::{
        shared::{
            minwindef::{BOOL, DWORD, WORD},
            windef::HWND,
            wtypesbase::{LPCOLESTR, LPOLESTR},
        },
        um::{
            minwinbase::WIN32_FIND_DATAW,
            objidl::{IPersist, IPersistVtbl},
            shtypes::{PCIDLIST_ABSOLUTE, PIDLIST_ABSOLUTE},
            unknwnbase::{IUnknown, IUnknownVtbl},
            winnt::{HRESULT, LPCWSTR, LPWSTR},
        },
        DEFINE_GUID, RIDL,
    };

    DEFINE_GUID! {CLSID_ShellLink,
    0x0002_1401, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}

    DEFINE_GUID! {IID_IShellLinkW,
    0x0002_14f9, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}
    RIDL! {
        #[uuid(0x0002_14f9, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46)]
        interface IShellLinkW(IShellLinkWVtbl): IUnknown(IUnknownVtbl) {
            fn GetPath(
                pszFile: LPWSTR,
                cch: i32,
                pfd: &WIN32_FIND_DATAW,
                fFlags: DWORD,
            ) -> HRESULT,
            fn GetIDList(
                ppidl: &PIDLIST_ABSOLUTE,
            ) -> HRESULT,
            fn SetIDList(
                pidl: PCIDLIST_ABSOLUTE,
            ) -> HRESULT,
            fn GetDescription(
                pszName: LPWSTR,
                cch: i32,
            ) -> HRESULT,
            fn SetDescription(
                pszName: LPCWSTR,
            ) -> HRESULT,
            fn GetWorkingDirectory(
                pszDir: LPWSTR,
                cch: i32,
            ) -> HRESULT,
            fn SetWorkingDirectory(
                pszDir: LPCWSTR,
            ) -> HRESULT,
            fn GetArguments(
                pszArgs: LPWSTR,
                cch: i32,
            ) -> HRESULT,
            fn SetArguments(
                pszArgs: LPCWSTR,
            ) -> HRESULT,
            fn GetHotkey(
                pwHotkey: &WORD,
            ) -> HRESULT,
            fn SetHotkey(
                wHotkey: WORD,
            ) -> HRESULT,
            fn GetShowCmd(
                piShowCmd: &i32,
            ) -> HRESULT,
            fn SetShowCmd(
                iShowCmd: i32,
            ) -> HRESULT,
            fn GetIconLocation(
                pszIconPath: LPWSTR,
                cch: i32,
                piIcon: &i32,
            ) -> HRESULT,
            fn SetIconLocation(
                pszIconPath: LPCWSTR,
                iIcon: i32,
            ) -> HRESULT,
            fn SetRelativePath(
                pszPathRel: LPCWSTR,
                dwReserved: DWORD,
            ) -> HRESULT,
            fn Resolve(
                hwnd: HWND,
                fFlags: DWORD,
            ) -> HRESULT,
            fn SetPath(
                pszFile: LPCWSTR,
            ) -> HRESULT,
        }
    }

    DEFINE_GUID! {IID_IPersistFile,
    0x0000_010b, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}
    RIDL! {
        #[uuid(0x0000_010b, 0x0000, 0x0000, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46)]
        interface IPersistFile(IPersistFileVtbl): IPersist(IPersistVtbl) {
            fn IsDirty() -> HRESULT,
            fn Load(
                pszFileName: LPCOLESTR,
                dwMode: DWORD,
            ) -> HRESULT,
            fn Save(
                pszFileName: LPCOLESTR,
                fRemember: BOOL,
            ) -> HRESULT,
            fn SaveCompleted(
                pszFileName: LPCOLESTR,
            ) -> HRESULT,
            fn GetCurFile(
                ppszFileName: &LPOLESTR,
            ) -> HRESULT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win32_string() {
        assert_eq!(
            vec![
                72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, 32, 55358, 56623, 55358,
                56623, 0
            ],
            win32_string("Hello World! \u{1F92F}\u{1F92F}")
        );
    }

    #[test]
    fn test_get_self_location() {
        let path = get_self_location().expect("Should never fail");
        let split: Vec<&str> = path.split(r"\").collect();

        // We don't care about the path that leads to the wedge directory
        let split: Vec<&str> = split.into_iter().skip_while(|a| *a != "wedge").collect();

        // Is path to build used for testing
        assert_eq!(
            vec![
                "wedge",
                "target",
                if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                },
                "deps",
                split[4]
            ],
            split
        );
    }

    #[test]
    fn test_get_temp_location() {
        let path = get_temp_location().expect("Should never fail");
        let split: Vec<&str> = path.split(r"\").collect();

        // Starts with some valid drive letter
        let drive: Vec<char> = split[0].chars().collect();
        assert!(drive[0].is_alphabetic());
        assert!(drive[0].is_uppercase());
        assert_eq!(vec![drive[0], ':'], drive);

        // Path looks correct
        assert_eq!(
            vec![split[0], "Users", split[2], "AppData", "Local", "Temp", ""],
            split
        );
    }

    #[test]
    fn test_get_local_install_location() {
        let path = get_local_install_location().expect("Should never fail");
        let split: Vec<&str> = path.split(r"\").collect();

        // Starts with some valid drive letter
        let drive: Vec<char> = split[0].chars().collect();
        assert!(drive[0].is_alphabetic());
        assert!(drive[0].is_uppercase());
        assert_eq!(vec![drive[0], ':'], drive);

        // Path looks correct
        assert_eq!(vec![split[0], "Users", split[2], "AppData", "Local"], split);
    }

    #[test]
    fn test_get_user_start_menu_location() {
        let path = get_user_start_menu_location().expect("Should never fail");
        let split: Vec<&str> = path.split(r"\").collect();

        // Starts with some valid drive letter
        let drive: Vec<char> = split[0].chars().collect();
        assert!(drive[0].is_alphabetic());
        assert!(drive[0].is_uppercase());
        assert_eq!(vec![drive[0], ':'], drive);

        // Path looks correct
        assert_eq!(
            vec![
                split[0],
                "Users",
                split[2],
                "AppData",
                "Roaming",
                "Microsoft",
                "Windows",
                "Start Menu",
                "Programs"
            ],
            split
        );
    }
}
