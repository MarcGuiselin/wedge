use super::{win32::*, *};
use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, File},
    io::{Error, ErrorKind, Write},
    path::Path,
    ptr::null_mut,
};
use winapi::um::{
    libloaderapi::{FindResourceW, GetModuleHandleW, LoadResource, LockResource, SizeofResource},
    winnt::LPWSTR,
    winuser::RT_RCDATA,
};
use winreg::{enums::*, RegKey};

// General install settings
pub const INSTALL_FOLDER: &str = "Wedge";
pub const APP_ID: &str = "Wedge";
pub const APP_DESC: &str =
    "Open web links normally forced to open in Microsoft Edge in your default web browser.";
pub const APP_NAME: &str = "Wedge";
pub const STEP_INTERVAL: u32 = 100;
pub const STEP_COUNT: isize = 6;

// Resource names
pub const BINARY_NAME: &str = "wedge.exe";
pub const UNINSTALLER_NAME: &str = "wedge uninstaller.exe";
pub const LICENSE_NAME: &str = "LICENSE";

// Resource Ids
const BINARY_RESOURCE: LPWSTR = MAKEINTRESOURCE!(301);
const UNINSTALLER_RESOURCE: LPWSTR = MAKEINTRESOURCE!(302);
const LICENSE_RESOURCE: LPWSTR = MAKEINTRESOURCE!(303);

// Install size is computed during runtime
static mut INSTALL_SIZE: u32 = 0;

/// Install Wedge step by step
pub fn install(step: usize) -> Result<String, Error> {
    let install_path = Path::new(&get_local_install_location()?).join(&INSTALL_FOLDER);
    let install_path_string = format!("\"{}\"", install_path.to_str().unwrap());
    let binary_path = install_path.join(&BINARY_NAME);
    let uninstaller_path = install_path.join(&UNINSTALLER_NAME);
    let binary_path_string = format!("\"{}\"", binary_path.to_str().unwrap());
    let uninstaller_path_string = format!("\"{}\"", uninstaller_path.to_str().unwrap());

    // Run different steps
    Ok(match step {
        // Unpack resources into installation folder
        1 => unsafe {
            let mut ret = vec![format!(
                "Created install location `{}`",
                install_path.display()
            )];

            // Create install directory
            create_dir_all(&install_path)?;

            // Module instance
            let hinstance = GetModuleHandleW(null_mut());

            // Reset install size
            INSTALL_SIZE = 0;

            // Unpack files and place in install directory iteratively
            for (name, id) in &[
                (BINARY_NAME, BINARY_RESOURCE),
                (UNINSTALLER_NAME, UNINSTALLER_RESOURCE),
                (LICENSE_NAME, LICENSE_RESOURCE),
            ] {
                // Unpack embedded binary resource
                // Code derived from https://blog.kowalczyk.info/article/zy/embedding-binary-resources-on-windows.html
                let res = FindResourceW(hinstance, *id, RT_RCDATA);
                let res_handle = LoadResource(hinstance, res);
                if res_handle.is_null() {
                    return Err(Error::new(ErrorKind::Other, "Failed unpacking resource"));
                }
                let res_data = LockResource(res_handle) as *const u8;
                let res_size = SizeofResource(hinstance, res) as usize;
                let input_as_u32 = std::slice::from_raw_parts(res_data, res_size).to_vec();

                // Write to install location
                let path = install_path.join(name);
                let mut file = File::create(&path)?;
                file.write_all(&input_as_u32)?;

                // Add unpacked file size to total install size
                INSTALL_SIZE += res_size as u32;

                // Notify successful extraction of file
                ret.push(format!("Unpacked `{}`", path.to_str().unwrap()));
            }
            ret.join("\n")
        },
        // Registry keys
        // https://docs.microsoft.com/en-us/windows/win32/shell/app-registration
        2 => {
            // Install for this user locally
            let software = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software")?;

            // Register app path as per https://docs.microsoft.com/en-us/windows/win32/shell/app-registration
            let (app_path, _) = software.create_subkey(
                Path::new(r"Microsoft\Windows\CurrentVersion\App Paths").join(&BINARY_NAME),
            )?;
            app_path.set_value("", &binary_path_string)?;
            app_path.set_value("Path", &install_path_string)?;

            // Create AppId
            let (class, _) = software.create_subkey(Path::new("Classes").join(&APP_ID))?;
            class.set_value("", &"URL: Microsoft Edge Protocol Deflector")?;
            class.set_value("URL Protocol", &"")?;
            let (default_icon, _) = class.create_subkey("DefaultIcon")?;
            default_icon.set_value("", &binary_path_string)?;
            let (command, _) = class.create_subkey(r"shell\open\command")?;
            command.set_value("", &format!("{} \"%1\"", &binary_path_string))?;

            // Registering AppId
            let registered_applications =
                software.open_subkey_with_flags(r"RegisteredApplications", KEY_ALL_ACCESS)?;
            registered_applications.set_value(
                APP_ID,
                &Path::new(r"Software\Clients")
                    .join(&APP_ID)
                    .join("Capabilities")
                    .to_str()
                    .unwrap(),
            )?;

            // Register Uninstaller
            let (uninstall, _) = software.create_subkey(
                Path::new(r"Microsoft\Windows\CurrentVersion\Uninstall").join(APP_ID),
            )?;
            uninstall.set_value("DisplayIcon", &binary_path_string)?;
            uninstall.set_value("DisplayName", &APP_NAME)?;
            uninstall.set_value("DisplayVersion", &"0.1.0")?;
            uninstall.set_value("EstimatedSize", &(unsafe { INSTALL_SIZE } / 1024u32))?;
            uninstall.set_value("InstallLocation", &install_path_string)?;
            uninstall.set_value("NoModify", &1u32)?;
            uninstall.set_value("NoRepair", &1u32)?;
            uninstall.set_value("Publisher", &env!("CARGO_PKG_AUTHORS"))?;
            uninstall.set_value("UninstallString", &uninstaller_path_string)?;
            String::from("Registered application")
        }
        // Create start menu link
        3 => {
            create_link(
                &Path::new(&get_user_start_menu_location()?)
                    .join(&format!("{}.lnk", APP_NAME))
                    .to_str()
                    .unwrap(),
                &binary_path_string,
                "Wedge - The simple Open-Source Edge Deflector",
            )?;
            String::from("Created start menu shortcut")
        }
        // Register IFEO
        4 => {
            let (ifeo, _) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\msedge.exe")?;
            ifeo.set_value("UseFilter", &1u32)?;

            let (filter, _) = ifeo.create_subkey(r"0")?;
            filter.set_value("Debugger", &binary_path_string)?;
            filter.set_value(
                "FilterFullPath",
                &r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            )?;

            String::from("Registered IFEO")
        }
        // Open link to confirm success
        5 => {
            // Open a link that would typically be opened up in edge
            shell_execute("microsoft-edge:https://www.bing.com/#notify-redirect-extension-successful-wedge-install");
            String::from(
                "All steps completed successfully! You may now close this installer.\nWedge can \
                 be easily uninstalled in windows Apps & Features",
            )
        }
        _ => String::new(),
    })
}

/// Uninstall Wedge
pub fn uninstall() -> Result<(), Error> {
    // Unregister IFEO
    RegKey::predef(HKEY_LOCAL_MACHINE).delete_subkey_all(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\msedge.exe",
    )?;

    // Try to delete install location. This will only succede when the original
    // uninstaller stops running.
    let install_path = Path::new(&get_local_install_location()?).join(&INSTALL_FOLDER);

    // Try deleting source file until it succeeds
    while remove_dir_all(&install_path).is_err() {
        // std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Delete shortcut if it's still there
    let _ =
        remove_file(Path::new(&get_user_start_menu_location()?).join(&format!("{}.lnk", APP_NAME)));

    // Remove local install registry entries
    let software = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software")?;

    // Remove registering of binary
    software.delete_subkey_all(
        Path::new(r"Microsoft\Windows\CurrentVersion\App Paths").join(&BINARY_NAME),
    )?;

    // Unregister AppId and "microsoft-edge:" url association
    software.delete_subkey_all(Path::new("Classes").join(&APP_ID))?;

    // Unregister uninstaller
    software
        .delete_subkey_all(Path::new(r"Microsoft\Windows\CurrentVersion\Uninstall").join(APP_ID))?;

    Ok(())
}
