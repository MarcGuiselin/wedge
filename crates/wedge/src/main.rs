// Don't use console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod uri;
use std::{env, path::PathBuf, process::Command};
use wedge_lib::{
    browser::{get_default_browser, Browser},
    install::{MSEDGE_PATH, MSEDGE_PROXY_PATH},
    win32::shell_execute,
};

/// Entry
#[cfg(windows)]
fn main() {
    let is_running_as_debugger = env::args().nth(1).unwrap_or_default() == MSEDGE_PATH;

    if is_running_as_debugger {
        let edge_args: Vec<String> = env::args().into_iter().skip(2).collect();

        let default_browser = get_default_browser().unwrap_or(Browser::Unknown);

        let deflected_url = if default_browser == Browser::Edge {
            None
        } else {
            edge_args.iter().find_map(|a| uri::parse_ms_edge_url(&a))
        };

        match deflected_url {
            Some(url) => {
                // Open the url with the system's default browser
                shell_execute(&url);
            }
            None => {
                let mut default_cwd = PathBuf::from(MSEDGE_PATH);
                default_cwd.pop();

                // Launch edge from the same cwd
                let cwd = std::env::current_dir().unwrap_or(default_cwd.into());

                // Get path to edge executable through alternate execution path that avoids ifeo
                let edge_alt_path = MSEDGE_PROXY_PATH;

                // Call msedge with the same args it would have originally been called with
                Command::new(edge_alt_path)
                    .args(edge_args)
                    .current_dir(cwd)
                    .spawn()
                    .expect("failed to execute process");
            }
        }
    }
}
