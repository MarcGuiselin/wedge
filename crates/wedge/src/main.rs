// Don't use console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod uri;
use std::{env, path::PathBuf, process::Command};
use wedge_lib::{
    browser::{get_default_browser, Browser},
    win32::{get_self_location, shell_execute},
};

/// Entry
#[cfg(windows)]
fn main() {
    let is_running_as_debugger = env::args().nth(1).unwrap_or_default()
        == r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe";

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
                // Get path to edge executable through junction point adjacent to this executable
                let mut edge_alt_path = PathBuf::from(&get_self_location().unwrap()).to_owned();
                edge_alt_path.pop();
                let edge_alt_path = edge_alt_path.join(&r"Edge\msedge.exe");

                // Call msedge with the same args it would have originally been called with
                Command::new(edge_alt_path)
                    .args(edge_args)
                    .current_dir(r"C:\Program Files (x86)\Microsoft\Edge\Application\")
                    .spawn()
                    .expect("failed to execute process");
            }
        }
    }
}
