// Don't use console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod uri;
use std::env;
use wedge_lib::win32::shell_execute;

/// Entry
#[cfg(windows)]
fn main() {
    for arg in env::args() {
        // Parse url from ms schema
        if let Some(url) = uri::parse_ms_edge_url(&arg) {
            // Open the url with the system's default browser
            shell_execute(&url);
        }
    }
}
