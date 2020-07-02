// Don't use console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ack_dialog;
mod install_dialog;
use std::{env, process::exit};
use wedge_lib::install::{install, STEP_COUNT};

#[cfg(windows)]
fn main() {
    // Run a silent install
    if env::args().any(|a| a == r"/quiet" || a == r"-quiet") {
        for i in 0..STEP_COUNT {
            match install(i as usize) {
                Ok(msg) => println!(
                    "Step {}/{}\n    {}",
                    i,
                    STEP_COUNT - 1,
                    msg.replace("\n", "\n    ")
                ),
                Err(e) => panic!("Error on step {}/{} {}", i, STEP_COUNT - 1, e),
            };
        }
    }
    // Run attended install
    else {
        // User must acknowledge prompt
        if ack_dialog::display().unwrap() {
            // Install dialog will handle install
            install_dialog::display().unwrap();
        }
    }
    exit(0);
}
